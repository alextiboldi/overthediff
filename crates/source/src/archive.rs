//! The zip provider (E04-S2): a zip archive's entries as a source's files and folders.
//!
//! Opening reads only the end records and the central directory, so listing and stat never touch
//! an entry's data, and each file reports the CRC-32 the archive stores. Reading one entry
//! streams it, checked against that CRC.
//!
//! - Folders the archive holds no entry for (`docs/` for `docs/a.txt`) are listed all the same,
//!   with the archive's own modified time.
//! - Times are the entries' DOS times, which have no zone: they are local time, in 2 s steps.
//! - Attributes are the DOS attribute byte every entry stores. A zip keeps no owner or group, so
//!   the Unix mode some entries carry isn't reported.
//! - An entry whose name climbs out with `..`, or sits more than 256 folders deep, is listed at
//!   the root under its whole name, with an error, so it can't be reached or copied. A file that is also another entry's folder is listed
//!   as the folder, with an error.
//!
//! ponytail: read-only, every write refused, until E04-S2-T2 writes archives. Symbolic links in an
//! archive are listed as files holding their target, and archives inside the archive aren't
//! expandable: both wait for the stories that need them. craft-zip doesn't yet expose an entry's
//! stored CRC, time or attributes, or an entry reader that owns its archive, so this module reads
//! the directory and entries through the `zip` crate it wraps (#132).

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, SystemTime};

use chrono::{Local, NaiveDate};
use zip::read::{ZipArchiveMetadata, ZipFileEntry, ZipReadOptions};

use crate::local::os_error;
use crate::memory::segments;
use crate::{Attributes, Capabilities, DataSource, Delete, ErrorKind, Item, ItemKind, Links, Permissions, SourceError};

/// Most entries an archive may declare, checked before the directory is indexed so a small file
/// can't make the index hold millions. ponytail: a fixed number; an option if real archives need more.
const MAX_ENTRIES: u64 = 1 << 20;
/// Most bytes of central directory and end records indexing may read.
const MAX_METADATA_BYTES: u64 = 256 << 20;
/// How much of an entry each hand-over between the reading thread and its reader holds.
const CHUNK: usize = 64 << 10;
/// Most folders an entry may sit in. A deeper one is listed at the root with an error, as copying
/// it would take a call per folder.
const MAX_DEPTH: usize = 256;
/// Where [`ZipSource::nodes`] keeps the root.
const ROOT: usize = 0;
/// The DOS attribute every folder has.
const DIRECTORY: u32 = 0x10;

trait Storage: Read + Seek + Send {}
impl<T: Read + Seek + Send> Storage for T {}

/// A zip archive as a read-only source. Paths are entry names, relative to the archive's root.
pub struct ZipSource {
    /// What error messages call the archive.
    name: String,
    storage: Arc<Mutex<Box<dyn Storage>>>,
    metadata: Arc<ZipArchiveMetadata>,
    /// Every file and folder; the root is [`ROOT`].
    nodes: Vec<Node>,
}

struct Node {
    item: Item,
    /// A file's index in the directory.
    entry: Option<usize>,
    /// A folder's children by name, as indexes into [`ZipSource::nodes`]. Names, not paths, so a
    /// deeply nested entry costs its name's length, not its length times its depth.
    children: BTreeMap<String, usize>,
}

impl ZipSource {
    /// Opens the archive at `path`. A file that isn't a zip archive, or whose directory is
    /// damaged, is an I/O error naming it.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, SourceError> {
        let path = path.into();
        let name = path.display().to_string();
        let file = File::open(&path).map_err(|e| os_error(&path, &e))?;
        let modified = file.metadata().and_then(|m| m.modified()).map_err(|e| os_error(&path, &e))?;
        Self::from_reader(name, file, modified)
    }

    /// An archive held in `reader`, `name` in messages. Folders it holds no entry for get `modified`.
    pub fn from_reader(name: String, mut reader: impl Read + Seek + Send + 'static, modified: SystemTime) -> Result<Self, SourceError> {
        let damaged = |e: &dyn std::fmt::Display| SourceError::new(ErrorKind::Io, format!("{name} isn't a readable zip archive: {e}"));
        craft_zip::check_entry_count_seekable(&mut reader, MAX_ENTRIES).map_err(|e| damaged(&e))?;
        let (bounded, indexing) = craft_zip::MetadataReader::new(reader, MAX_METADATA_BYTES).map_err(|e| damaged(&e))?;
        let archive = zip::ZipArchive::new(bounded).map_err(|e| damaged(&e))?;
        // Entry reads come after indexing, anywhere in the archive.
        indexing.finish();
        let metadata = archive.metadata();
        let storage: Box<dyn Storage> = Box::new(archive.into_inner());
        let root = Node::folder(String::new(), modified);
        let mut source = Self { name: name.clone(), storage: Arc::new(Mutex::new(storage)), metadata: Arc::clone(&metadata), nodes: vec![root] };
        for index in 0..metadata.len() {
            let entry = metadata.entry(index).map_err(|e| damaged(&e))?;
            source.add(index, &entry, modified);
        }
        Ok(source)
    }

    /// Puts an entry, and the folders above it, in the index.
    fn add(&mut self, index: usize, entry: &ZipFileEntry<'_>, fallback: SystemTime) {
        let raw = entry.name().map_or_else(|_| String::from_utf8_lossy(entry.name_raw()).into_owned(), |name| name.into_owned());
        let modified = entry.last_modified().and_then(local_time).unwrap_or(fallback);
        let names: Vec<&str> = raw.split(['/', '\\']).filter(|name| !name.is_empty() && *name != ".").collect();
        let Some((last, parents)) = names.split_last() else { return };
        let unreachable = if names.contains(&"..") {
            Some("points outside the archive")
        } else if names.len() > MAX_DEPTH {
            Some("is nested deeper than folders can be copied")
        } else {
            None
        };
        if let Some(why) = unreachable {
            let error = SourceError::new(ErrorKind::NotFound, format!("{raw} in {} {why}", self.name));
            let item = Item { error: Some(error), ..file_item(raw.clone(), entry, modified) };
            self.insert(ROOT, &raw, Node { item, entry: None, children: BTreeMap::new() });
            return;
        }
        let mut parent = ROOT;
        for name in parents {
            parent = self.folder(parent, name, fallback);
        }
        if entry.is_dir() {
            let at = self.folder(parent, last, modified);
            if let Some(node) = self.nodes.get_mut(at) {
                node.item.modified = Some(modified);
            }
        } else {
            let node = Node { item: file_item((*last).to_owned(), entry, modified), entry: Some(index), children: BTreeMap::new() };
            self.insert(parent, last, node);
        }
    }

    /// Folder `name` in `parent`, made if it isn't there. A file in its place becomes the folder,
    /// with an error, so what is inside stays reachable.
    fn folder(&mut self, parent: usize, name: &str, modified: SystemTime) -> usize {
        let Some(&at) = self.nodes.get(parent).and_then(|node| node.children.get(name)) else {
            return self.insert(parent, name, Node::folder(name.to_owned(), modified));
        };
        if let Some(node) = self.nodes.get_mut(at).filter(|node| node.item.kind == ItemKind::File) {
            *node = Node::folder(name.to_owned(), modified);
            node.item.error = Some(clash(&self.name, name));
        }
        at
    }

    /// Adds `node` as `name` in `parent`. When something is already there, it stays and carries
    /// the error.
    fn insert(&mut self, parent: usize, name: &str, node: Node) -> usize {
        if let Some(&at) = self.nodes.get(parent).and_then(|folder| folder.children.get(name)) {
            if let Some(existing) = self.nodes.get_mut(at) {
                existing.item.error = Some(clash(&self.name, name));
            }
            return at;
        }
        let at = self.nodes.len();
        self.nodes.push(node);
        if let Some(folder) = self.nodes.get_mut(parent) {
            folder.children.insert(name.to_owned(), at);
        }
        at
    }

    fn node(&self, path: &Path) -> Result<&Node, SourceError> {
        let mut node = self.nodes.get(ROOT);
        for name in segments(path)? {
            node = node.and_then(|folder| folder.children.get(&name)).and_then(|&at| self.nodes.get(at));
        }
        node.ok_or_else(|| SourceError::new(ErrorKind::NotFound, format!("{} isn't in {}", path.display(), self.name)))
    }

    fn read_only(&self) -> SourceError {
        SourceError::new(ErrorKind::AccessDenied, format!("{} is read-only: changing what an archive holds isn't supported yet", self.name))
    }
}

impl Node {
    fn folder(name: String, modified: SystemTime) -> Self {
        let item = Item {
            name,
            kind: ItemKind::Folder,
            size: 0,
            modified: Some(modified),
            created: None,
            attributes: Some(Attributes::Windows(DIRECTORY)),
            link_target: None,
            crc32: None,
            container: false,
            error: None,
        };
        Self { item, entry: None, children: BTreeMap::new() }
    }
}

fn file_item(name: String, entry: &ZipFileEntry<'_>, modified: SystemTime) -> Item {
    Item {
        name,
        kind: ItemKind::File,
        size: entry.size(),
        modified: Some(modified),
        created: None,
        attributes: Some(Attributes::Windows(entry.external_attributes() & 0xFF)),
        link_target: None,
        // An encrypted entry may store 0 in its place (AE-2).
        crc32: (!entry.encrypted()).then(|| entry.crc32()),
        container: false,
        error: None,
    }
}

fn clash(archive: &str, name: &str) -> SourceError {
    SourceError::new(ErrorKind::Io, format!("{archive} holds more than one entry named {name}"))
}

/// A DOS time as local time. A date that doesn't exist, or a time a clock change skipped, is `None`.
fn local_time(time: zip::DateTime) -> Option<SystemTime> {
    let date = NaiveDate::from_ymd_opt(time.year().into(), time.month().into(), time.day().into())?;
    let naive = date.and_hms_opt(time.hour().into(), time.minute().into(), time.second().into())?;
    Some(naive.and_local_timezone(Local).earliest()?.into())
}

impl DataSource for ZipSource {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            read_only: true,
            case_sensitive: true,
            recycle_bin: false,
            timestamp_resolution: Duration::from_secs(2),
            windows_attributes: false,
            unix_permissions: false,
        }
    }

    fn list(&self, folder: &Path, _: Links) -> Result<Vec<Item>, SourceError> {
        let node = self.node(folder)?;
        if node.item.kind != ItemKind::Folder {
            return Err(SourceError::new(ErrorKind::Io, format!("{} isn't a folder", folder.display())));
        }
        Ok(node.children.values().filter_map(|&at| self.nodes.get(at)).map(|child| child.item.clone()).collect())
    }

    fn stat(&self, path: &Path, _: Links) -> Result<Item, SourceError> {
        Ok(self.node(path)?.item.clone())
    }

    fn open_read(&self, path: &Path) -> Result<Box<dyn Read + Send>, SourceError> {
        let node = self.node(path)?;
        if let Some(error) = &node.item.error {
            return Err(error.clone());
        }
        let Some(index) = node.entry else {
            return Err(SourceError::new(ErrorKind::Io, format!("{} is a folder", path.display())));
        };
        let (metadata, mut storage) = (Arc::clone(&self.metadata), Shared { storage: Arc::clone(&self.storage), position: 0 });
        let (chunks, receiver) = mpsc::sync_channel(2);
        // `zip`'s entry reader borrows the archive, and the contract's reader owns what it reads, so
        // a thread of its own reads the entry. It ends when the entry does, or when the reader is dropped.
        let reading = move || {
            let entry = metadata.entry(index).map_err(io::Error::other);
            let mut data = match entry.and_then(|entry| entry.with_reader(&mut storage, ZipReadOptions::new()).map_err(io::Error::other)) {
                Ok(data) => data,
                Err(e) => return drop(chunks.send(Err(e))),
            };
            loop {
                let mut chunk = vec![0; CHUNK];
                let read = data.read(&mut chunk).map(|n| chunk.truncate(n));
                let end = read.as_ref().map_or(true, |()| chunk.is_empty());
                // An empty chunk is the end of the entry.
                if chunks.send(read.map(|()| chunk)).is_err() || end {
                    return;
                }
            }
        };
        thread::Builder::new()
            .name("zip entry".to_owned())
            .spawn(reading)
            .map_err(|e| SourceError::new(ErrorKind::Io, format!("reading {} from {}: {e}", path.display(), self.name)))?;
        Ok(Box::new(EntryReader { chunks: receiver, chunk: Vec::new(), at: 0, ended: false }))
    }

    fn write(&self, _: &Path, _: &mut dyn Read) -> Result<(), SourceError> {
        Err(self.read_only())
    }

    fn create_folder(&self, _: &Path) -> Result<(), SourceError> {
        Err(self.read_only())
    }

    fn delete(&self, _: &Path, _: Delete) -> Result<(), SourceError> {
        Err(self.read_only())
    }

    fn rename(&self, _: &Path, _: &str) -> Result<(), SourceError> {
        Err(self.read_only())
    }

    fn set_modified(&self, _: &Path, _: SystemTime) -> Result<(), SourceError> {
        Err(self.read_only())
    }

    fn set_permissions(&self, _: &Path, _: Permissions) -> Result<(), SourceError> {
        Err(self.read_only())
    }
}

/// The archive's storage with a position of its own, so entry reads on several threads can share it.
struct Shared {
    storage: Arc<Mutex<Box<dyn Storage>>>,
    position: u64,
}

impl Read for Shared {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut storage = self.storage.lock().unwrap_or_else(PoisonError::into_inner);
        storage.seek(SeekFrom::Start(self.position))?;
        let read = storage.read(buf)?;
        self.position += read as u64;
        Ok(read)
    }
}

impl Seek for Shared {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let start = match to {
            SeekFrom::Start(at) => at,
            SeekFrom::End(_) => self.storage.lock().unwrap_or_else(PoisonError::into_inner).seek(to)?,
            SeekFrom::Current(by) => {
                self.position.checked_add_signed(by).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before the start"))?
            }
        };
        self.position = start;
        Ok(start)
    }
}

/// One entry's content, as the reading thread hands it over.
struct EntryReader {
    chunks: Receiver<io::Result<Vec<u8>>>,
    chunk: Vec<u8>,
    at: usize,
    ended: bool,
}

impl Read for EntryReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        while self.at == self.chunk.len() {
            if self.ended || buf.is_empty() {
                return Ok(0);
            }
            match self.chunks.recv() {
                Ok(chunk) => {
                    self.chunk = chunk?;
                    self.at = 0;
                    self.ended = self.chunk.is_empty();
                }
                // The thread stopped without handing over the end.
                Err(_) => return Err(io::Error::other("the entry's reader stopped before its end")),
            }
        }
        let n = buf.len().min(self.chunk.len() - self.at);
        buf[..n].copy_from_slice(&self.chunk[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}
