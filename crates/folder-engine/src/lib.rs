//! OverTheDiff's folder comparison engine (E03-S2). It reads two trees only through the
//! data-source contract in `otd-source`, and its tests use that crate's `MemorySource` as the
//! fixture for both sides. It has no UI and runs without a window.

mod align;
mod content;
mod scan;
mod status;

pub use align::{Aligned, Placed, align, align_flat, case_sensitive};
pub use content::{Compared, ContentOptions, ContentResult, ContentTest, Side, compare};
pub use scan::{Folder, ScanEvent, ScanState, TreeScan};
pub use status::{Criteria, Status, quick_test};
