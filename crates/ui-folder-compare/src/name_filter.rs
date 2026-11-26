//! The view's name filter (E08-S3-T1): its masks go into the comparison's visibility hook through
//! `otd_folder_engine::NameFilter`, so excluded items aren't drawn, counted or coloured into their
//! folders. The toolbar edit (E08-S3-T2) and the Name Filters tab (E08-S3-T3) set it.

use otd_folder_engine::{MaskListError, NameFilter};

use crate::FolderCompare;

impl FolderCompare {
    /// The name masks in use, as an A.1 mask list such as `*.cs;-obj\;-bin\`; empty for none.
    pub fn name_filter(&self) -> &str {
        &self.name_filter
    }

    /// Filters the view by `masks`, an A.1 mask list, and compares again. A list that doesn't
    /// compile changes nothing and says why.
    pub fn set_name_filter(&mut self, masks: &str) -> Result<(), MaskListError> {
        NameFilter::new(masks, [true, true])?;
        self.name_filter = masks.to_owned();
        self.compare();
        self.forget_hidden();
        Ok(())
    }

    /// The filter compiled with each side's case rule.
    pub(crate) fn compiled_name_filter(&self, case_sensitive: [bool; 2]) -> NameFilter {
        // `set_name_filter` only keeps lists that compile, and compiling doesn't depend on case.
        NameFilter::new(&self.name_filter, case_sensitive).unwrap_or_default()
    }
}
