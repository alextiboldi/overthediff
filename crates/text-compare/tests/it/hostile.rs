//! The decoder never panics or hangs on damaged bytes (AGENTS.md hostile-input sweeps).

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_text_compare::decode;

#[test]
fn decoding_never_panics_or_hangs_on_truncated_or_damaged_bytes() {
    // Each seed reaches another branch: marked UTF-16 (where cuts leave odd lengths and lone
    // surrogates), UTF-16 without a mark, UTF-8 with mixed endings, and code-page text.
    let mut marked = vec![0xFF, 0xFE];
    marked.extend("a\u{1F600}\r\nb\r".encode_utf16().flat_map(u16::to_le_bytes));
    let unmarked: Vec<u8> = "plain text\nmore".encode_utf16().flat_map(u16::to_be_bytes).collect();
    for (name, seed) in
        [("marked UTF-16", marked), ("UTF-16", unmarked), ("UTF-8", "é\r\nx\ry\n".as_bytes().to_vec()), ("code page", b"caf\xE9\r\n".to_vec())]
    {
        hammer(&HammerConfig::new(name, 2_000), &seed, |data| {
            let _ = decode(data, 1252);
            let _ = decode(data, 932);
        });
    }
}
