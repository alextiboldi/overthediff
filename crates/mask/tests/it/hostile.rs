//! Masks come from settings, sessions and the command line, so compiling and matching one never
//! panics or hangs, whatever the bytes.

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_mask::{Case, MaskList, Wildcard};

#[test]
fn compiling_and_matching_a_damaged_mask_never_panics_or_hangs() {
    let seed = b"[!a-c][[]?*x*.";
    let name = "d[ex.y";
    // A sweep from a seed that doesn't compile would only exercise the first error.
    assert!(Wildcard::new(std::str::from_utf8(seed).unwrap(), Case::Sensitive).is_ok_and(|w| w.matches("d[ex")), "the seed compiles and matches");

    hammer(&HammerConfig::new("wildcard masks", 2_000), seed, |data| {
        for case in [Case::Sensitive, Case::Insensitive] {
            if let Ok(w) = Wildcard::new(&String::from_utf8_lossy(data), case) {
                let _ = w.matches(name);
            }
        }
    });
}

#[test]
fn compiling_and_evaluating_a_damaged_mask_list_never_panics_or_hangs() {
    let seed = br"*.pas;-Backup\;.\W*\...\[a-c]?.txt; -\;";
    let path = ["W", "x", "b1.txt"];
    assert!(
        MaskList::new(std::str::from_utf8(seed).unwrap(), Case::Sensitive).is_ok_and(|m| m.includes_file(&path)),
        "the seed compiles and matches"
    );

    hammer(&HammerConfig::new("mask lists", 2_000), seed, move |data| {
        if let Ok(masks) = MaskList::new(&String::from_utf8_lossy(data), Case::Insensitive) {
            let _ = (masks.includes_file(&path), masks.folder(&path));
        }
    });
}
