//! The hostile-input sweep: a pattern is user input, so damaged patterns, compiled and run on a
//! sample text, never panic or hang.

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_regex::Regex;

#[test]
fn damaged_patterns_never_panic_or_hang() {
    let seed = r"(?i)^\s*(\w+|[^\d\-x-z]{2,5}?)\x41\x{263A}(a(?-i)b)*?\1\t\Z|end;?$";
    let text = "  Begin A\u{263A}ab\nend;\r\nabcabc\t";
    let compiled = Regex::new(seed).unwrap_or_else(|e| panic!("the seed compiles: {e}"));
    assert!(compiled.matches(text).all(|m| m.is_ok()), "the seed runs");

    hammer(&HammerConfig::new("regex patterns", 2_000), seed.as_bytes(), |data| {
        if let Ok(regex) = Regex::new(&String::from_utf8_lossy(data)) {
            regex.matches(text).for_each(drop);
        }
    });
}
