//! Text or binary, from a file's first bytes (story E02-S1, task T7 decision). craft-libs follow-up:
//! #40.

/// How many leading bytes [`is_binary`] looks at.
pub const SAMPLE_LEN: usize = 8 * 1024;

/// Whether `content` is binary: its first [`SAMPLE_LEN`] bytes hold a NUL character or aren't
/// valid text. A UTF-16 BOM (little- or big-endian) makes them UTF-16, otherwise they must be
/// UTF-8, with or without a BOM. A character cut in two at byte [`SAMPLE_LEN`] still counts as
/// text, so a caller may pass just the first [`SAMPLE_LEN`] bytes of a longer file. An empty file
/// is text.
///
/// So UTF-16 without a BOM, UTF-32 and legacy code-page text are binary here: a text format picks
/// them up by mask or by name instead.
pub fn is_binary(content: &[u8]) -> bool {
    // ponytail: a file of exactly SAMPLE_LEN bytes ending inside a character reads as text too.
    let cut = content.len() >= SAMPLE_LEN;
    let sample = content.get(..SAMPLE_LEN).unwrap_or(content);
    if let Some(rest) = sample.strip_prefix(&[0xFF, 0xFE]) {
        !utf16_text(rest, u16::from_le_bytes, cut)
    } else if let Some(rest) = sample.strip_prefix(&[0xFE, 0xFF]) {
        !utf16_text(rest, u16::from_be_bytes, cut)
    } else {
        let rest = sample.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(sample);
        let valid = match std::str::from_utf8(rest) {
            Ok(_) => true,
            // `error_len() == None`: the bytes end inside a character.
            Err(e) => cut && e.error_len().is_none(),
        };
        !valid || rest.contains(&0)
    }
}

fn utf16_text(bytes: &[u8], unit: fn([u8; 2]) -> u16, cut: bool) -> bool {
    let (mut units, odd) = bytes.as_chunks::<2>();
    if !odd.is_empty() {
        return false;
    }
    // A high surrogate whose pair the sample cut off.
    if cut
        && let [rest @ .., last] = units
        && (0xD800..0xDC00).contains(&unit(*last))
    {
        units = rest;
    }
    char::decode_utf16(units.iter().map(|u| unit(*u))).all(|c| c.is_ok_and(|c| c != '\0'))
}
