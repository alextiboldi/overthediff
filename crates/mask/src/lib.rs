//! OverTheDiff's file masks (feature catalogue A.1): wildcards for single names, and `;`-separated
//! mask lists evaluated over tree paths. UI-free: it builds and tests without a window.

mod list;
mod wildcard;

pub use list::{Folder, MaskList, MaskListError};
pub use wildcard::{Case, Wildcard, WildcardError};
