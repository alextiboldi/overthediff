//! Content tests (E03-S2-T5): a file pair's content compared by CRC-32 or byte by byte, and the
//! result folded into the quick tests' status under the skip and override options.

use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use otd_source::{DataSource, ErrorKind, Item, ItemKind, SourceError};

use crate::{Aligned, Criteria, Status, quick_test};

/// How a file pair's content is compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentTest {
    /// CRC-32 of each side: the source's own CRC when it provides one, otherwise computed.
    Crc,
    /// Byte by byte.
    Binary,
}

/// A content test's result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentResult {
    BinarySame,
    BinaryDifferences,
}

/// Which content test runs, and how its result meets the quick tests'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentOptions {
    /// `None` runs no content test.
    pub test: Option<ContentTest>,
    /// Run the content test only for pairs the quick tests call different.
    pub skip_if_quick_same: bool,
    /// A content match makes the row same even when the quick tests disagree.
    pub override_quick_results: bool,
}

impl Default for ContentOptions {
    /// Override is on by default (F-8.1). No content test and no skipping are *unspecified*
    /// defaults: the view's options decide what a session runs.
    fn default() -> Self {
        Self { test: None, skip_if_quick_same: false, override_quick_results: true }
    }
}

/// One side of a pair: the source and the folder, relative to its root, holding the row's item.
pub type Side<'a> = (&'a dyn DataSource, &'a Path);

/// A row's status after the quick and content tests, and the content result when one ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compared {
    pub status: Status,
    pub content: Option<ContentResult>,
}

/// Runs the quick tests, then the content test when the options call for one.
///
/// - A content test runs only for two files (a followed link counts as its target), never for
///   an orphan, an error or a folder.
/// - Files of different sizes have binary differences without being read.
/// - A content match makes the row same with override on, and leaves the quick status with it
///   off. A content mismatch makes a quick-same row different and leaves any other quick status
///   (newer keeps its direction). *Unspecified*: the story only says what a match does.
/// - A read that fails makes the row an error that keeps the message, with no content result.
pub fn compare(row: &Aligned, criteria: &Criteria, options: &ContentOptions, left: Side<'_>, right: Side<'_>) -> Compared {
    let quick = quick_test(row, criteria);
    let (Some(test), Some(left_item), Some(right_item)) = (options.test, &row.left, &row.right) else {
        return Compared { status: quick, content: None };
    };
    let files = left_item.kind == ItemKind::File && right_item.kind == ItemKind::File;
    let eligible = files && !matches!(quick, Status::Error(_)) && !(options.skip_if_quick_same && quick == Status::Same);
    if !eligible {
        return Compared { status: quick, content: None };
    }
    let result = match content_test(test, (left.0, left.1, left_item), (right.0, right.1, right_item)) {
        Ok(result) => result,
        Err(error) => return Compared { status: Status::Error(error), content: None },
    };
    let status = match result {
        ContentResult::BinarySame if options.override_quick_results => Status::Same,
        ContentResult::BinaryDifferences if quick == Status::Same => Status::Different,
        _ => quick,
    };
    Compared { status, content: Some(result) }
}

type File<'a> = (&'a dyn DataSource, &'a Path, &'a Item);

fn content_test(test: ContentTest, left: File<'_>, right: File<'_>) -> Result<ContentResult, SourceError> {
    if left.2.size != right.2.size {
        return Ok(ContentResult::BinaryDifferences);
    }
    let same = match test {
        ContentTest::Crc => crc32(left.0, left.1, left.2)? == crc32(right.0, right.1, right.2)?,
        ContentTest::Binary => same_bytes(left, right)?,
    };
    Ok(if same { ContentResult::BinarySame } else { ContentResult::BinaryDifferences })
}

/// The CRC-32 of `item`, a file in `folder` (relative to the source's root): the source's own as-is
/// when it provides one, otherwise the content's, read through the source.
pub fn crc32(source: &dyn DataSource, folder: &Path, item: &Item) -> Result<u32, SourceError> {
    if let Some(crc) = item.crc32 {
        return Ok(crc);
    }
    let file = (source, folder, item);
    let mut reader = open(file)?;
    let mut hasher = crc32fast::Hasher::new();
    loop {
        let buf = reader.fill_buf().map_err(|e| read_error(file, e))?;
        if buf.is_empty() {
            return Ok(hasher.finalize());
        }
        hasher.update(buf);
        let n = buf.len();
        reader.consume(n);
    }
}

fn same_bytes(left: File<'_>, right: File<'_>) -> Result<bool, SourceError> {
    let (mut l, mut r) = (open(left)?, open(right)?);
    loop {
        let a = l.fill_buf().map_err(|e| read_error(left, e))?;
        let b = r.fill_buf().map_err(|e| read_error(right, e))?;
        if a.is_empty() || b.is_empty() {
            return Ok(a.is_empty() && b.is_empty());
        }
        let n = a.len().min(b.len());
        if a.get(..n) != b.get(..n) {
            return Ok(false);
        }
        l.consume(n);
        r.consume(n);
    }
}

fn open(file: File<'_>) -> Result<BufReader<Box<dyn Read + Send>>, SourceError> {
    Ok(BufReader::with_capacity(64 * 1024, file.0.open_read(&file.1.join(&file.2.name))?))
}

fn read_error(file: File<'_>, error: std::io::Error) -> SourceError {
    SourceError::new(ErrorKind::Io, format!("couldn't read {}: {error}", file.1.join(&file.2.name).display()))
}
