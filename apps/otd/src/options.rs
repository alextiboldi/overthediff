//! The program options features contribute to the Options dialog's pages. A feature adds its
//! `otd_ui::OptionSpec`s here, one line each (`options.extend(feature::options())`). Options naming
//! the same page share it; two with one id fail the test below and the app's start.

use otd_ui::OptionSpec;

pub fn options() -> Vec<OptionSpec> {
    let mut options = Vec::new();
    options.extend(otd_ui::theme::options());
    options
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_two_contributed_options_share_an_id() {
        let shell = otd_ui::Shell::new(Vec::new()).with_options(super::options(), Default::default());
        assert_eq!(shell.err(), None);
    }
}
