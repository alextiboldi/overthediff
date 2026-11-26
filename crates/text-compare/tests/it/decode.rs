//! Encoding and line-ending detection.

use otd_text_compare::{Encoding, LineEnding, LineEndings, decode, decode_as, encode};

const TEXT: &str = "café\nnaïve\n";

fn utf16(text: &str, unit: fn(u16) -> [u8; 2]) -> Vec<u8> {
    text.encode_utf16().flat_map(unit).collect()
}

fn texts(bytes: &[u8]) -> Vec<String> {
    decode(bytes, 1252).lines.into_iter().map(|line| line.text).collect()
}

#[test]
fn a_byte_order_mark_names_the_encoding_and_stays_out_of_the_text() {
    let cases = [
        ([&[0xEF, 0xBB, 0xBF][..], TEXT.as_bytes()].concat(), Encoding::Utf8 { bom: true }),
        ([&[0xFF, 0xFE][..], &utf16(TEXT, u16::to_le_bytes)].concat(), Encoding::Utf16Le { bom: true }),
        ([&[0xFE, 0xFF][..], &utf16(TEXT, u16::to_be_bytes)].concat(), Encoding::Utf16Be { bom: true }),
    ];
    for (bytes, encoding) in cases {
        assert_eq!(decode(&bytes, 1252).encoding, encoding);
        assert_eq!(texts(&bytes), ["café", "naïve"], "{encoding:?}");
    }
}

#[test]
fn valid_utf8_without_a_mark_is_utf8() {
    assert_eq!(decode(TEXT.as_bytes(), 1252).encoding, Encoding::Utf8 { bom: false });
    assert_eq!(texts(TEXT.as_bytes()), ["café", "naïve"]);
}

#[test]
fn utf16_without_a_mark_is_told_apart_by_where_its_zero_bytes_sit() {
    for (bytes, encoding) in
        [(utf16(TEXT, u16::to_le_bytes), Encoding::Utf16Le { bom: false }), (utf16(TEXT, u16::to_be_bytes), Encoding::Utf16Be { bom: false })]
    {
        assert_eq!(decode(&bytes, 1252).encoding, encoding);
        assert_eq!(texts(&bytes), ["café", "naïve"], "{encoding:?}");
    }
}

#[test]
fn other_bytes_decode_in_the_code_page_given() {
    // "café" in Windows-1252; 0xE9 is a different letter in Windows-1251.
    let bytes = b"caf\xE9\n";
    assert_eq!(decode(bytes, 1252).encoding, Encoding::CodePage(1252));
    assert_eq!(texts(bytes), ["café"]);
    let cyrillic = decode(bytes, 1251);
    assert_eq!(cyrillic.encoding, Encoding::CodePage(1251));
    assert_ne!(cyrillic.lines, decode(bytes, 1252).lines);
}

#[test]
fn a_code_page_with_no_table_falls_back_to_one_that_has_and_names_it() {
    // 65001 is UTF-8 as a code page, which has no byte table.
    let decoded = decode(b"caf\xE9", 65001);
    assert!(matches!(decoded.encoding, Encoding::CodePage(used) if used != 65001), "{:?}", decoded.encoding);
}

#[test]
fn an_east_asian_code_page_decodes_lead_and_trail_bytes_together() {
    // "日本" in Shift-JIS (932): two two-byte characters, neither UTF-8 nor UTF-16.
    let decoded = decode(b"\x93\xFA\x96\x7B", 932);
    assert_eq!(decoded.encoding, Encoding::CodePage(932));
    assert_eq!(decoded.lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>(), ["日本"]);
}

#[test]
fn each_line_ending_style_is_detected() {
    for (text, style) in [("a\r\nb\r\n", LineEndings::Windows), ("a\rb\r", LineEndings::Mac), ("a\nb\n", LineEndings::Unix)] {
        let decoded = decode(text.as_bytes(), 1252);
        assert_eq!(decoded.endings, style, "{text:?}");
        assert_eq!(decoded.lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>(), ["a", "b"], "{text:?}");
    }
}

#[test]
fn two_kinds_of_break_are_mixed_but_a_last_line_without_one_is_not() {
    assert_eq!(decode(b"a\r\nb\nc", 1252).endings, LineEndings::Mixed);
    assert_eq!(decode(b"a\nb", 1252).endings, LineEndings::Unix);
}

#[test]
fn a_file_without_a_break_has_no_style() {
    assert_eq!(decode(b"", 1252).endings, LineEndings::None);
    assert!(decode(b"", 1252).lines.is_empty(), "an empty file has no lines");
    let one = decode(b"a", 1252);
    assert_eq!(one.endings, LineEndings::None);
    assert_eq!(one.lines.len(), 1);
}

#[test]
fn a_final_break_ends_the_last_line_without_adding_an_empty_one() {
    let lines = decode(b"a\r\n\r\n", 1252).lines;
    assert_eq!(lines.len(), 2, "a blank line, then nothing");
    assert!(lines.iter().all(|line| line.ending == LineEnding::CrLf));
}

#[test]
fn an_override_decodes_windows_1252_text_that_detection_took_for_utf8() {
    // "CafÃ©" in Windows-1252 is also valid UTF-8, for "Café".
    let bytes = b"Caf\xC3\xA9\n";
    assert_eq!(decode(bytes, 1252).encoding, Encoding::Utf8 { bom: false }, "detection is fooled");
    let forced = decode_as(bytes, Encoding::CodePage(1252));
    assert_eq!(forced.encoding, Encoding::CodePage(1252));
    assert_eq!(forced.lines[0].text, "CafÃ©");
}

#[test]
fn an_override_keeps_its_own_byte_order_mark_out_of_the_text_so_the_file_saves_back_unchanged() {
    for (bytes, encoding) in [
        ([&[0xEF, 0xBB, 0xBF][..], TEXT.as_bytes()].concat(), Encoding::Utf8 { bom: false }),
        ([&[0xFF, 0xFE][..], &utf16(TEXT, u16::to_le_bytes)].concat(), Encoding::Utf16Le { bom: false }),
        (utf16(TEXT, u16::to_be_bytes), Encoding::Utf16Be { bom: true }),
    ] {
        let forced = decode_as(&bytes, encoding);
        assert_eq!(forced.lines[0].text, "café", "{encoding:?}");
        assert_eq!(encode(&forced.lines, forced.encoding).as_deref(), Ok(&bytes[..]), "{encoding:?}");
    }
}
