//! The launch-argument handlers. A feature registers its `otd_launch::Handler<Shell>` here, one line
//! each. The first switch argument on the command line decides which switch handler claims it;
//! the order here only matters between two handlers registered under the same switch name.

use otd_launch::Registry;

use crate::Shell;
use crate::quick_compare::QuickCompare;

pub fn registry() -> Registry<Shell> {
    let mut registry = Registry::new();
    registry.register(QuickCompare("qc"));
    registry.register(QuickCompare("quickcompare"));
    registry
}
