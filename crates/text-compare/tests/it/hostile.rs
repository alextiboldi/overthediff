//! The decoder, with and without an override, and the encoder on what it decodes, never panic or
//! hang on damaged bytes, nor does finding damaged text (AGENTS.md hostile-input sweeps).

use craft_testkit::hostile::{HammerConfig, hammer};
use otd_text_compare::{Direction, Encoding, FindOptions, Finder, Line, LineEnding, Pos, Selection, Side, Sides, decode, decode_as, encode};

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
            for page in [1252, 932] {
                let decoded = decode(data, page);
                let _ = encode(&decoded.lines, decoded.encoding);
            }
            // An override decodes whatever it is given (E07-S10-T2).
            for encoding in
                [Encoding::Utf8 { bom: false }, Encoding::Utf16Le { bom: false }, Encoding::Utf16Be { bom: false }, Encoding::CodePage(949)]
            {
                let decoded = decode_as(data, encoding);
                let _ = encode(&decoded.lines, decoded.encoding);
            }
        });
    }
}

#[test]
fn finding_never_panics_or_hangs_on_damaged_text_to_find() {
    let lines: Vec<Line> = ["é☺ apple Apples", "", "  (x)*\\1 e\u{301}"].map(|text| Line { text: text.to_owned(), ending: LineEnding::Lf }).into();
    let seed = r"(\w+)\s*\1|é☺?^$.*";
    hammer(&HammerConfig::new("text to find", 2_000), seed.as_bytes(), move |data| {
        let text = String::from_utf8_lossy(data);
        for (regex, whole_words) in [(false, false), (true, false), (true, true)] {
            let options = FindOptions { regex, whole_words, wrap: true, sides: Sides::Both, ..FindOptions::default() };
            if let Ok(finder) = Finder::new(&text, options) {
                let _ = finder.find([&lines, &lines], Side::Right, Selection::caret(Pos { line: 9, column: 99 }), Direction::Backward, None);
            }
        }
    });
}
