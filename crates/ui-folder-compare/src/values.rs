//! The Ext and CRC columns' values and a folder's Size (E08-S6-T3).
//!
//! - **Ext** is a file's extension: the name after its last dot, none for a name that starts with
//!   its only dot (`.profile`). A folder has none (*unspecified*, decided here: `v1.2` is a name,
//!   not a type); an archive opened as a folder keeps its own.
//! - **Size** of a folder is the total of the files below it on that side, blank until everything
//!   below it is listed ([`Row::total_size`]). With background scanning off, that is once the
//!   folder and every folder below it are opened (*Open Subfolders* or *Expand All*).
//! - **CRC** is a file's CRC-32 in hexadecimal. A CRC the source already holds (an archive's) is
//!   used as-is; otherwise the file is read on a worker thread of its side the first time its
//!   cell is in view, and the cell says *Reading…* until it is done, or *Can't read*. A value is
//!   kept for the file's size and modified time, so a Refresh that finds the file changed reads it
//!   again, and a rescan starts over.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::SystemTime;

use otd_core::i18n::{t, trn};
use otd_folder_engine::{Row, TreeSide, crc32};
use otd_source::{DataSource, Item, ItemKind, SourceError};

use crate::{BYTE, BYTES, Column, SIZE};

// English keys, drawn only through `t`.
pub const EXT: &str = "Ext";
pub const CRC: &str = "CRC";
/// A CRC cell while its file is read.
pub const READING: &str = "Reading…";
/// A CRC cell whose file couldn't be read.
pub const CANT_READ: &str = "Can't read";

/// The Ext column's cell.
pub(crate) fn ext(item: &Item) -> Option<String> {
    let file = item.kind == ItemKind::File || item.container;
    let ext = Path::new(&item.name).extension()?.to_string_lossy();
    (file && !ext.is_empty()).then(|| ext.into_owned())
}

/// The CRC column's cell: the CRC the source holds. [`extra`] reads the others.
pub(crate) fn source_crc(item: &Item) -> Option<String> {
    item.crc32.map(hex)
}

fn hex(crc: u32) -> String {
    format!("{crc:08X}")
}

/// Where a cell is: its row, the side, that side's folder holding the item, that side's CRCs, and
/// whether the cell is in view.
pub(crate) struct At<'a> {
    pub row: &'a Row,
    pub side: TreeSide,
    pub folder: Option<&'a Path>,
    pub crcs: Option<&'a mut Crcs>,
    pub in_view: bool,
}

/// A cell's text: its column's own `cell`, or else a folder's total size, or a file's CRC that the
/// source doesn't hold.
pub(crate) fn text(column: &Column, item: &Item, at: At<'_>) -> Option<String> {
    (column.cell)(item).or_else(|| extra(column.caption, item, at))
}

fn extra(caption: &str, item: &Item, at: At<'_>) -> Option<String> {
    match caption {
        SIZE => at.row.total_size(at.side).map(|size| trn(size, BYTE, BYTES)),
        CRC if item.kind == ItemKind::File => {
            let crc = at.crcs?.get(at.folder?, item, at.in_view)?;
            Some(match crc {
                Crc::Reading => t(READING).to_owned(),
                Crc::Read(crc) => hex(*crc),
                Crc::Failed => t(CANT_READ).to_owned(),
            })
        }
        _ => None,
    }
}

/// A file's CRC, as far as it is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Crc {
    Reading,
    Read(u32),
    Failed,
}

/// What a CRC was read for: the file's size and modified time.
type Stamp = (u64, Option<SystemTime>);

fn stamp(item: &Item) -> Stamp {
    (item.size, item.modified)
}

/// A read asked for: the file's folder from the base, and the file.
type Job = (PathBuf, Item);

/// A read finished: the file's path from the base, what it was read for, and its CRC.
type Done = (PathBuf, Stamp, Result<u32, SourceError>);

/// One side's CRCs, read from its source on a worker thread that stops once this is dropped.
pub(crate) struct Crcs {
    source: Arc<dyn DataSource>,
    known: HashMap<PathBuf, (Stamp, Crc)>,
    worker: Option<(Sender<Job>, Receiver<Done>)>,
}

impl Crcs {
    pub(crate) fn new(source: Arc<dyn DataSource>) -> Self {
        Self { source, known: HashMap::new(), worker: None }
    }

    /// The CRC of `item` in `folder` as the file is now. When none is known, `start` sets it reading
    /// and `Reading` is returned; without `start`, nothing.
    pub(crate) fn get(&mut self, folder: &Path, item: &Item, start: bool) -> Option<&Crc> {
        let path = folder.join(&item.name);
        let current = self.known.get(&path).is_some_and(|(at, _)| *at == stamp(item));
        if !current {
            if !start {
                return None;
            }
            let crc = if self.send(folder, item) { Crc::Reading } else { Crc::Failed };
            self.known.insert(path.clone(), (stamp(item), crc));
        }
        self.known.get(&path).map(|(_, crc)| crc)
    }

    /// Hands `item` to the worker, starting it the first time. `false` when no worker can take it.
    fn send(&mut self, folder: &Path, item: &Item) -> bool {
        if self.worker.is_none() {
            let (jobs, queue) = mpsc::channel::<Job>();
            let (finished, done) = mpsc::channel();
            let source = Arc::clone(&self.source);
            let spawned = thread::Builder::new().name("otd-crc".into()).spawn(move || {
                for (folder, item) in queue {
                    let crc = crc32(&*source, &folder, &item);
                    if finished.send((folder.join(&item.name), stamp(&item), crc)).is_err() {
                        return;
                    }
                }
            });
            if spawned.is_err() {
                return false;
            }
            self.worker = Some((jobs, done));
        }
        self.worker.as_ref().is_some_and(|(jobs, _)| jobs.send((folder.to_path_buf(), item.clone())).is_ok())
    }

    /// Takes in the reads finished since the last call. One for a file that has changed since it
    /// was asked for is dropped: the file's newer read is on its way.
    pub(crate) fn take(&mut self) {
        let Some((_, done)) = &self.worker else { return };
        while let Ok((path, at, crc)) = done.try_recv() {
            if let Some((stamp, known)) = self.known.get_mut(&path)
                && *stamp == at
            {
                *known = crc.map_or(Crc::Failed, Crc::Read);
            }
        }
    }

    /// Whether a read is still under way.
    pub(crate) fn reading(&self) -> bool {
        self.known.values().any(|(_, crc)| *crc == Crc::Reading)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use otd_source::{Capabilities, Links, MemorySource};

    use super::*;

    fn source(file: &str, content: &str) -> MemorySource {
        let caps = Capabilities {
            read_only: true,
            case_sensitive: true,
            recycle_bin: false,
            timestamp_resolution: Duration::ZERO,
            windows_attributes: false,
            unix_permissions: false,
        };
        MemorySource::new(caps).file(file, content, SystemTime::UNIX_EPOCH)
    }

    fn item(source: &MemorySource, name: &str) -> Item {
        source.list(Path::new(""), Links::Report).unwrap().into_iter().find(|item| item.name == name).unwrap()
    }

    /// Asks for `item`'s CRC until the worker has read it.
    fn read(crcs: &mut Crcs, item: &Item) -> Crc {
        // The limit only stops a test that would hang.
        for _ in 0..1000 {
            crcs.take();
            match crcs.get(Path::new(""), item, true) {
                Some(Crc::Reading) => std::thread::sleep(Duration::from_millis(2)),
                Some(crc) => return *crc,
                None => break,
            }
        }
        panic!("the CRC was never read");
    }

    #[test]
    fn a_crc_the_source_holds_shows_as_is_without_a_read() {
        let source = source("f.zip", "not what the CRC says").crc32("f.zip", 0x1234_ABCD);
        assert_eq!(source_crc(&item(&source, "f.zip")), Some(hex(0x1234_ABCD)));
    }

    /// The CRC-32 check value: the CRC of `123456789` (ISO-HDLC, the zip and PNG CRC).
    #[test]
    fn a_crc_the_source_does_not_hold_is_read_from_the_content() {
        let source = source("f.txt", "123456789");
        let item = item(&source, "f.txt");
        let mut crcs = Crcs::new(Arc::new(source));
        assert_eq!(read(&mut crcs, &item), Crc::Read(0xCBF4_3926));
        assert!(!crcs.reading());
    }

    #[test]
    fn a_file_changed_since_its_crc_was_read_is_read_again() {
        let source = source("f.txt", "123456789");
        let item = item(&source, "f.txt");
        let mut crcs = Crcs::new(Arc::new(source));
        let first = read(&mut crcs, &item);
        let changed = Item { size: item.size + 1, ..item };
        assert_eq!(crcs.get(Path::new(""), &changed, true), Some(&Crc::Reading), "not {first:?}");
    }

    #[test]
    fn an_unreadable_file_s_crc_fails_rather_than_reads_forever() {
        let source = source("f.txt", "x").unreadable("f.txt");
        let item = item(&source, "f.txt");
        let mut crcs = Crcs::new(Arc::new(source));
        assert_eq!(read(&mut crcs, &item), Crc::Failed);
    }

    #[test]
    fn ext_is_the_part_after_a_file_s_last_dot_and_none_for_a_folder_or_dot_file() {
        let at = |name: &str, kind| ext(&Item { name: name.into(), kind, ..item(&source("x", ""), "x") });
        assert_eq!(at("archive.tar.gz", ItemKind::File).as_deref(), Some("gz"));
        assert_eq!([at(".profile", ItemKind::File), at("trailing.", ItemKind::File), at("v1.2", ItemKind::Folder)], [None, None, None]);
    }
}
