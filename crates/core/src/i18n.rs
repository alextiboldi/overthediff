//! Translation lookup for captions, labels and messages (E01-S1-T13). The English text in the code
//! is the key: [`t`] returns its translation in the interface language, or the English itself when
//! there is none. A missing translation never fails anything.
//!
//! Lookup, fallback (`pt-br` → `pt` → English), plurals and `{name}` filling are craft-libs'
//! `craft-i18n`. The catalogs are storytold/craft-i18n's `locales/<locale>/common.tsv` and
//! `overthediff.tsv`, embedded at build time from the optional `CRAFT_I18N_DIR` (see `build.rs`);
//! without it the build is English only. Feature pull requests add English only.
//!
//! The language is per thread, so parallel UI tests can't change each other's. The app sets the
//! system's on the UI thread at start-up ([`set_language`]); other threads look up in English.
// ponytail: per thread; a process-wide language once a background task needs translated text off
// the UI thread, or a language setting exists.

use std::cell::Cell;
use std::sync::LazyLock;

pub use craft_i18n::{Embedded, I18n, Lang, fmt};

include!(concat!(env!("OUT_DIR"), "/craft_i18n.rs"));

/// The embedded catalogs. Right-to-left locales are fallbacks only, not offered, until the window
/// can be mirrored.
static EMBEDDED: LazyLock<I18n> = LazyLock::new(|| {
    let offered: Vec<&str> =
        CRAFT_I18N.iter().map(|(code, ..)| *code).filter(|code| craft_i18n::direction(code) == craft_i18n::Direction::Ltr).collect();
    let (i18n, problems) = I18n::new(CRAFT_I18N, &offered);
    for problem in problems {
        eprintln!("otd: translation catalog: {problem}");
    }
    i18n
});

thread_local! {
    /// The catalogs and language [`t`] reads on this thread; `None` is English.
    static CURRENT: Cell<Option<(&'static I18n, Lang)>> = const { Cell::new(None) };
}

/// The catalogs built into the app.
pub fn embedded() -> &'static I18n {
    &EMBEDDED
}

/// Looks text up in `lang` of `catalogs` on this thread from now on. The app passes
/// [`embedded`] and its [`I18n::system_lang`]; a test may pass catalogs of its own.
pub fn set_language(catalogs: &'static I18n, lang: Lang) {
    CURRENT.set(Some((catalogs, lang)));
}

fn current() -> (&'static I18n, Lang) {
    CURRENT.get().unwrap_or_else(|| (embedded(), Lang::EN))
}

/// `source`, English text, in the interface language, or `source` itself without a translation.
pub fn t(source: &str) -> &str {
    let (catalogs, lang) = current();
    catalogs.t(lang, source)
}

/// `value` with `decimals` digits in the unit template `unit` (`{n} s`), an English key, the number
/// written for the user's region.
pub fn measure(value: f64, decimals: usize, unit: &str) -> String {
    let (catalogs, lang) = current();
    catalogs.measure(lang, value, decimals, unit)
}

/// The wording for a count `n`, `one` or `other` (`{n} item failed:`, `{n} items failed:`), both
/// English keys, in the interface language's plural form, with `n` written for the user's region.
pub fn trn(n: u64, one: &str, other: &str) -> String {
    let (catalogs, lang) = current();
    catalogs.trn(lang, n, one, other)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogs(rows: &'static str) -> (&'static I18n, Lang) {
        let embedded: &'static Embedded = Box::leak(Box::new([("de", "", rows)]));
        let catalogs: &'static I18n = Box::leak(Box::new(I18n::new(embedded, &["de"]).0));
        (catalogs, catalogs.from_code("de").unwrap_or(Lang::EN))
    }

    #[test]
    fn the_language_is_per_thread() {
        let (catalogs, de) = catalogs("\tHome\tStart\n");
        set_language(catalogs, de);
        let elsewhere = std::thread::spawn(|| t("Home").to_owned()).join().unwrap_or_default();
        assert_eq!(elsewhere, "Home", "another thread still reads English");
        assert_ne!(t("Home"), elsewhere);
    }
}
