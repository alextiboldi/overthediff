//! A `;`-separated mask list evaluated over paths in a tree (feature catalogue A.1, story E02-S1).
//!
//! - `f` includes files named f, and `p\` includes folders named p and their subfolders.
//! - `-f` excludes files and `-p\` excludes folders, with everything in them.
//! - `p\f` requires f directly inside a folder named p, at any depth. `.\f` requires f in the base
//!   folder. A `...` part matches zero or more folders, so `p\...\f` finds f anywhere under p.
//! - The base folder is always included, and so are the files directly in it unless a file mask
//!   rules them out. A folder that no folder mask includes is shown only when something inside it
//!   is: [`Folder::IfContainsIncluded`].
//!
//! Story E02-S1 records these choices, which A.1 leaves open:
//! - A path mask without a leading `.\` may start at any depth, so `Windows\*.txt` and
//!   `...\Windows\*.txt` select the same files.
//! - A file is included only if it passes the file masks *and* sits in an included folder.
//! - An exclude wins over any include. With no include masks of a kind, everything of that kind not
//!   excluded is included.
//! - Spaces around a token are dropped, and empty tokens are skipped. A token whose name part is
//!   empty (`-`, `\`) matches nothing.
//!
//! Paths come as their names, relative to the base folder, so `\` and `/` in a name never split it.

use std::fmt;

use crate::{Case, Wildcard, WildcardError};

/// A compiled mask list. Build it once with [`MaskList::new`] and ask it about many paths.
#[derive(Clone, Debug, Default)]
pub struct MaskList {
    files: Rules,
    folders: Rules,
}

#[derive(Clone, Debug, Default)]
struct Rules {
    include: Vec<PathMask>,
    exclude: Vec<PathMask>,
}

/// One token's path: its parts, matched against a path's names from the base folder down.
#[derive(Clone, Debug)]
struct PathMask(Vec<Part>);

#[derive(Clone, Debug)]
enum Part {
    Name(Wildcard),
    /// `...`, or the implied start of a mask without `.\`: zero or more folders.
    AnyFolders,
}

/// How a folder fares against the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Folder {
    /// Shown, with whatever inside it the list includes.
    Included,
    /// Hidden, with everything inside it.
    Excluded,
    /// Shown only when something inside it is shown. Files directly in it are not included.
    IfContainsIncluded,
}

/// A token of the list that doesn't compile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskListError {
    /// The token as typed, without the spaces around it.
    pub mask: String,
    pub error: WildcardError,
}

impl fmt::Display for MaskListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.mask, self.error)
    }
}

impl std::error::Error for MaskListError {}

impl MaskList {
    /// Compiles `list`, such as `*.pas;-Backup\;.\Windows\*.txt`. An empty list includes everything.
    pub fn new(list: &str, case: Case) -> Result<Self, MaskListError> {
        let mut masks = Self::default();
        for token in list.split(';').map(str::trim).filter(|t| !t.is_empty()) {
            let (exclude, rest) = token.strip_prefix('-').map_or((false, token), |rest| (true, rest));
            let (folder, rest) = rest.strip_suffix('\\').map_or((false, rest), |rest| (true, rest));
            let mask = PathMask::new(rest, case).map_err(|error| MaskListError { mask: token.to_owned(), error })?;
            let rules = if folder { &mut masks.folders } else { &mut masks.files };
            if exclude { &mut rules.exclude } else { &mut rules.include }.push(mask);
        }
        Ok(masks)
    }

    /// Whether the file at `path`, its names from the base folder down to its own, is included.
    pub fn includes_file<S: AsRef<str>>(&self, path: &[S]) -> bool {
        let Some((_, parents)) = path.split_last() else { return false };
        // ponytail: re-checks the parent folder for every file. A caller that walks the tree can keep
        // each folder's verdict if mask lists or depths ever make that cost show.
        self.folder(parents) == Folder::Included
            && !self.files.exclude.iter().any(|m| m.matches(path))
            && (self.files.include.is_empty() || self.files.include.iter().any(|m| m.matches(path)))
    }

    /// How the folder at `path`, its names from the base folder down, fares. The base folder, an
    /// empty path, is always included.
    pub fn folder<S: AsRef<str>>(&self, path: &[S]) -> Folder {
        if path.is_empty() {
            Folder::Included
        } else if self.folders.exclude.iter().any(|m| m.matches_folder_or_ancestor(path)) {
            Folder::Excluded
        } else if self.folders.include.is_empty() || self.folders.include.iter().any(|m| m.matches_folder_or_ancestor(path)) {
            Folder::Included
        } else {
            Folder::IfContainsIncluded
        }
    }
}

impl PathMask {
    fn new(mask: &str, case: Case) -> Result<Self, WildcardError> {
        let mut names = mask.split('\\').peekable();
        let anchored = names.next_if_eq(&".").is_some();
        let mut parts = if anchored { Vec::new() } else { vec![Part::AnyFolders] };
        for name in names {
            parts.push(match name {
                "..." => Part::AnyFolders,
                name => Part::Name(Wildcard::new(name, case)?),
            });
        }
        Ok(Self(parts))
    }

    fn matches<S: AsRef<str>>(&self, path: &[S]) -> bool {
        self.table(path).last().copied().unwrap_or(false)
    }

    /// Whether the mask matches the folder at `path` or one of its ancestors below the base.
    fn matches_folder_or_ancestor<S: AsRef<str>>(&self, path: &[S]) -> bool {
        self.table(path).iter().skip(1).any(|&m| m)
    }

    /// `table[j]`: the mask matches the first `j` names of `path`. One pass answers for the path and
    /// every ancestor, and many `...` parts stay O(parts x names).
    fn table<S: AsRef<str>>(&self, path: &[S]) -> Vec<bool> {
        let mut row: Vec<bool> = std::iter::once(true).chain(path.iter().map(|_| false)).collect();
        for part in &self.0 {
            row = match part {
                // Once some names match, any number of folders after them may too.
                Part::AnyFolders => row
                    .iter()
                    .scan(false, |seen, &m| {
                        *seen |= m;
                        Some(*seen)
                    })
                    .collect(),
                Part::Name(w) => {
                    std::iter::once(false).chain(row.iter().zip(path).map(|(&before, name)| before && w.matches(name.as_ref()))).collect()
                }
            };
        }
        row
    }
}
