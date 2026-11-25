//! OverTheDiff's file-format registry (feature catalogue F-21.1, F-21.2, story E02-S1): the single
//! ordered list of formats that decides how each file is treated. UI-free: it builds and tests
//! without a window.
//!
//! - Features register the kinds they handle with [`Registry::register_kind`], then their formats
//!   with [`Registry::register`]. Formats keep registration order, so a feature's formats follow
//!   the built-ins registered before them.
//! - [`Registry::resolve`] scans from the top, and the first enabled format whose mask matches the
//!   file name wins. Masks follow A.1 through `otd-mask`, so a mask with no file include, such as
//!   the exclude-only `-*.bak` or the folder-only `Source\`, matches every other name.
//! - A format with an empty mask never resolves automatically, but [`Registry::by_name`] picks it,
//!   as it picks any format, enabled or not.
//!
//! Not here: the fallback for files no format matches (E02-S1-T7), editing and reordering formats
//! (E10-S1), and kind-specific properties, which the owning feature keeps by format name.

use std::fmt;

use otd_mask::MaskList;
pub use otd_mask::{Case, MaskListError};

/// How a viewer treats a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    Table,
    Hex,
    Picture,
    External,
}

/// One entry of the registry. Build it with [`Format::new`] and set the public fields as needed.
#[derive(Clone, Debug)]
pub struct Format {
    pub name: String,
    pub kind: Kind,
    pub description: String,
    /// A disabled format is skipped by [`Registry::resolve`].
    pub enabled: bool,
    /// Shipped with a feature rather than made by the user.
    pub built_in: bool,
    masks: String,
    /// One compiled list per case, so the data source's case is free at each lookup. `None` for an
    /// empty mask.
    compiled: Option<[MaskList; 2]>,
}

impl Format {
    /// An enabled, user-made format with no description. `masks` is a `;`-separated A.1 mask list,
    /// such as `*.txt;*.md`. One with no tokens (empty, or only `;` and spaces) is an empty mask.
    pub fn new(name: &str, kind: Kind, masks: &str) -> Result<Self, MaskListError> {
        let compiled = if masks.split(';').all(|t| t.trim().is_empty()) {
            None
        } else {
            Some([MaskList::new(masks, Case::Sensitive)?, MaskList::new(masks, Case::Insensitive)?])
        };
        Ok(Self { name: name.to_owned(), kind, description: String::new(), enabled: true, built_in: false, masks: masks.to_owned(), compiled })
    }

    /// The mask list as given.
    pub fn masks(&self) -> &str {
        &self.masks
    }

    /// Whether the mask matches `file_name`, compared with the data source's `case`. An empty mask
    /// matches nothing. The enabled flag is not consulted.
    pub fn matches(&self, file_name: &str, case: Case) -> bool {
        self.compiled.as_ref().is_some_and(|[sensitive, insensitive]| {
            let list = if case == Case::Sensitive { sensitive } else { insensitive };
            list.includes_file(&[file_name])
        })
    }
}

/// Why [`Registry::register`] refused a format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterError {
    /// No feature has registered the format's kind.
    KindNotRegistered(Kind),
    /// A format with this name is already registered, and picking by name must be unambiguous.
    DuplicateName(String),
}

impl fmt::Display for RegisterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KindNotRegistered(kind) => write!(f, "format kind {kind:?} is not registered"),
            Self::DuplicateName(name) => write!(f, "a format named {name} is already registered"),
        }
    }
}

impl std::error::Error for RegisterError {}

/// The single ordered list of formats, and the kinds features have registered.
#[derive(Clone, Debug, Default)]
pub struct Registry {
    kinds: Vec<Kind>,
    formats: Vec<Format>,
}

impl Registry {
    /// Makes `kind` available to formats. Registering it again changes nothing.
    pub fn register_kind(&mut self, kind: Kind) {
        if !self.has_kind(kind) {
            self.kinds.push(kind);
        }
    }

    pub fn has_kind(&self, kind: Kind) -> bool {
        self.kinds.contains(&kind)
    }

    /// Appends `format` below every format registered before it.
    pub fn register(&mut self, format: Format) -> Result<(), RegisterError> {
        if !self.has_kind(format.kind) {
            return Err(RegisterError::KindNotRegistered(format.kind));
        }
        if self.by_name(&format.name).is_some() {
            return Err(RegisterError::DuplicateName(format.name));
        }
        self.formats.push(format);
        Ok(())
    }

    /// Every format, top first.
    pub fn formats(&self) -> &[Format] {
        &self.formats
    }

    /// Enables or disables the format called `name`. False when there is none.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        self.formats.iter_mut().find(|f| f.name == name).map(|f| f.enabled = enabled).is_some()
    }

    /// The first enabled format, from the top, whose mask matches `file_name` with the data
    /// source's `case`. `None` when none does.
    pub fn resolve(&self, file_name: &str, case: Case) -> Option<&Format> {
        self.formats.iter().find(|f| f.enabled && f.matches(file_name, case))
    }

    /// The format called `name`, for a manual pick: its mask and enabled flag don't matter.
    pub fn by_name(&self, name: &str) -> Option<&Format> {
        self.formats.iter().find(|f| f.name == name)
    }
}
