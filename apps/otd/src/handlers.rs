//! The launch-argument handlers. A feature registers its `otd_launch::Handler<Shell>` here, one line
//! each; the order only matters among switch handlers matching the same command line.

use otd_launch::Registry;

use crate::Shell;

pub fn registry() -> Registry<Shell> {
    // ponytail: no feature handles arguments yet; E11 adds the first ones.
    Registry::new()
}
