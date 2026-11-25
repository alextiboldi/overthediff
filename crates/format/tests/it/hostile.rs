//! File contents are untrusted, so telling text from binary never panics or hangs, whatever the
//! bytes.

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_format::is_binary;

#[test]
fn telling_damaged_content_apart_never_panics_or_hangs() {
    // UTF-16 LE with a BOM and a surrogate pair, then a UTF-8 BOM and a two-byte character.
    let seed = b"\xff\xfea\x00\x3d\xd8\x00\xde\xef\xbb\xbf\xc3\xa9\x00";
    hammer(&HammerConfig::new("text or binary", 2_000), seed, |data| {
        let _ = is_binary(data);
    });
}
