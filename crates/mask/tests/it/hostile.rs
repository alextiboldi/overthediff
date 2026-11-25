//! Masks come from settings, sessions and the command line, so compiling and matching one never
//! panics or hangs, whatever the bytes.

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_mask::Wildcard;

#[test]
fn compiling_and_matching_a_damaged_mask_never_panics_or_hangs() {
    let seed = b"[!a-c][[]?*x*.";
    let name = "d[ex.y";
    // A sweep from a seed that doesn't compile would only exercise the first error.
    assert!(Wildcard::new(std::str::from_utf8(seed).unwrap()).is_ok_and(|w| w.matches("d[ex")), "the seed compiles and matches");

    hammer(&HammerConfig::new("wildcard masks", 2_000), seed, |data| {
        if let Ok(w) = Wildcard::new(&String::from_utf8_lossy(data)) {
            let _ = w.matches(name);
        }
    });
}
