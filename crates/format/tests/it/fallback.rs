//! The fallback for files no format matches (story E02-S1, task T7).

use otd_format::{CATCH_ALL, Case, Format, Kind, RegisterError, Registry, SAMPLE_LEN, is_binary};

/// The PNG signature and the start of an IHDR chunk, written out here.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00";

/// A Text format for `*.txt` and, when `hex`, a Hex format for `*.bin`.
fn registry(hex: bool) -> Registry {
    let mut registry = Registry::default();
    registry.register_kind(Kind::Text);
    registry.register(Format::new("Notes", Kind::Text, "*.txt").unwrap()).unwrap();
    if hex {
        registry.register_kind(Kind::Hex);
        registry.register(Format::new("Bytes", Kind::Hex, "*.bin").unwrap()).unwrap();
    }
    registry
}

fn fallback(registry: &Registry, content: &[u8]) -> String {
    registry.fallback(content).name.clone()
}

fn utf16(text: &str, bom: &[u8], unit: fn(u16) -> [u8; 2]) -> Vec<u8> {
    bom.iter().copied().chain(text.encode_utf16().flat_map(unit)).collect()
}

#[test]
fn a_png_with_an_unknown_extension_falls_back_to_hex_only_when_hex_is_registered() {
    let mut with_hex = registry(true);
    assert!(with_hex.resolve("picture.xyz", Case::Sensitive).is_none(), "no format matches");
    assert_eq!(with_hex.fallback(PNG).kind, Kind::Hex);
    assert_eq!(fallback(&registry(false), PNG), CATCH_ALL);

    with_hex.set_enabled("Bytes", false);
    assert_eq!(fallback(&with_hex, PNG), CATCH_ALL, "a disabled Hex format is skipped");
}

#[test]
fn utf8_text_falls_back_to_the_catch_all_text_format_even_when_hex_is_registered() {
    let registry = registry(true);
    assert_eq!(registry.catch_all().kind, Kind::Text);
    for text in ["naïve café\r\n\tdone\n", "\u{feff}with a BOM", ""] {
        assert_eq!(fallback(&registry, text.as_bytes()), CATCH_ALL, "{text:?}");
    }
}

#[test]
fn the_catch_all_never_resolves_by_mask_but_is_picked_by_name_and_keeps_its_name() {
    let mut registry = registry(false);
    assert!(registry.formats().iter().all(|f| f.name != CATCH_ALL), "outside the ordered list");
    for name in ["x.xyz", "noext", ""] {
        assert!(registry.resolve(name, Case::Sensitive).is_none(), "{name:?}");
    }
    assert!(registry.by_name(CATCH_ALL).is_some());
    let taken = Format::new(CATCH_ALL, Kind::Text, "*.md").unwrap();
    assert_eq!(registry.register(taken), Err(RegisterError::DuplicateName(CATCH_ALL.into())));
}

#[test]
fn utf16_with_a_bom_is_text_though_it_holds_nul_bytes() {
    let le = utf16("line 1\r\n\u{1F600}", &[0xFF, 0xFE], u16::to_le_bytes);
    let be = utf16("line 1\r\n\u{1F600}", &[0xFE, 0xFF], u16::to_be_bytes);
    assert!(le.contains(&0) && be.contains(&0));
    assert!(!is_binary(&le) && !is_binary(&be));
}

#[test]
fn a_nul_character_or_invalid_text_is_binary() {
    let cases: [(&str, Vec<u8>); 6] = [
        ("NUL in UTF-8", b"abc\0def".to_vec()),
        ("invalid UTF-8", b"caf\xe9 au lait".to_vec()),
        ("invalid UTF-8 after a BOM", b"\xef\xbb\xbf\xff".to_vec()),
        ("NUL in UTF-16", utf16("a\0b", &[0xFF, 0xFE], u16::to_le_bytes)),
        ("unpaired surrogate in UTF-16", [0xFF, 0xFE, 0x00, 0xD8, b'a', 0x00].to_vec()),
        ("odd byte count after a UTF-16 BOM", [0xFE, 0xFF, 0x00, b'a', 0x00].to_vec()),
    ];
    for (what, bytes) in cases {
        assert!(is_binary(&bytes), "{what}");
    }
}

#[test]
fn only_the_first_sample_is_looked_at() {
    let mut text = vec![b'a'; SAMPLE_LEN];
    text.extend_from_slice(b"\0\xff");
    assert!(!is_binary(&text));

    let mut nul_inside = vec![b'a'; SAMPLE_LEN];
    nul_inside[SAMPLE_LEN - 1] = 0;
    assert!(is_binary(&nul_inside));
}

#[test]
fn a_character_cut_at_the_end_of_the_sample_is_text_but_one_cut_at_the_end_of_a_short_file_is_not() {
    // "é" (2 bytes in UTF-8) and U+1F600 (a surrogate pair in UTF-16) straddle byte SAMPLE_LEN.
    let mut utf8 = vec![b'a'; SAMPLE_LEN - 1];
    utf8.extend_from_slice("é".as_bytes());
    let mut utf16le = utf16(&"a".repeat(SAMPLE_LEN / 2 - 2), &[0xFF, 0xFE], u16::to_le_bytes);
    utf16le.extend(utf16("\u{1F600}", &[], u16::to_le_bytes));
    // The same characters cut at the end of a short file.
    let short: [&[u8]; 2] = [b"ab\xc3", &[0xFF, 0xFE, b'a', 0x00, 0x3D, 0xD8]];
    for ((what, bytes), short) in [("UTF-8", utf8), ("UTF-16", utf16le)].into_iter().zip(short) {
        assert!(bytes.len() > SAMPLE_LEN, "{what} straddles the sample's end");
        assert!(!is_binary(&bytes), "{what} cut by the sample");
        assert!(!is_binary(&bytes[..SAMPLE_LEN]), "{what} sample alone");
        assert!(is_binary(short), "{what} short file ending inside a character");
    }
}

#[test]
fn utf16_without_a_bom_and_legacy_code_page_text_are_binary() {
    // Pins the decision: neither is detected, so a text format takes them by mask or by name.
    assert!(is_binary(&utf16("plain", &[], u16::to_le_bytes)[..]));
    assert!(is_binary(b"r\xe9sum\xe9 in Windows-1252"));
}
