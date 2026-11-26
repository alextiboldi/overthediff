//! Encoding and line-ending detection (E07-S1-T3): a byte-order mark first, then valid UTF-8, then
//! UTF-16 without a mark, and otherwise the system code page.

use craft_codepage::{SINGLE_BYTE_CODE_PAGES, multibyte_encoding_name, windows_byte};
use otd_text_engine::{Line, LineEnding};

/// The code page used when the one asked for is neither a single-byte Windows code page nor an
/// East Asian multibyte one.
pub const WESTERN: u16 = 1252;

/// How a file's bytes became text. Whether it had a byte-order mark is kept, so a save can write
/// the file back as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Utf8 {
        bom: bool,
    },
    Utf16Le {
        bom: bool,
    },
    Utf16Be {
        bom: bool,
    },
    /// A Windows code page by number: a single-byte one ([`SINGLE_BYTE_CODE_PAGES`]) or an East Asian
    /// multibyte one ([`craft_codepage::MULTIBYTE_CODE_PAGES`]: 932, 936, 949, 950).
    CodePage(u16),
}

/// A file's line-ending style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEndings {
    /// No line break at all: an empty file, or one line without a break.
    None,
    /// CR LF.
    Windows,
    /// CR.
    Mac,
    /// LF.
    Unix,
    /// More than one kind of break (*inferred*).
    Mixed,
}

/// A file's text, split into lines, with what was detected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    pub encoding: Encoding,
    pub endings: LineEndings,
    /// The last line ends with [`LineEnding::None`] unless the file ends with a break, which adds no
    /// empty line after it. An empty file has no lines.
    pub lines: Vec<Line>,
}

/// Decodes `bytes`, falling back to `code_page` when they are neither UTF-8 nor UTF-16. A code page
/// other than a single-byte Windows one or an East Asian multibyte one falls back to [`WESTERN`],
/// and the result names the page actually used. Never fails: bytes a mark or code page claims but
/// that don't decode become U+FFFD.
pub fn decode(bytes: &[u8], code_page: u16) -> Decoded {
    let (encoding, text) = detect(bytes, code_page);
    let lines = split(&text);
    Decoded { encoding, endings: endings(&lines), lines }
}

fn detect(bytes: &[u8], code_page: u16) -> (Encoding, String) {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return (Encoding::Utf8 { bom: true }, String::from_utf8_lossy(rest).into_owned());
    }
    // ponytail: FF FE 00 00 (UTF-32 LE) reads as UTF-16 LE; UTF-32 waits for a story that needs it.
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return (Encoding::Utf16Le { bom: true }, utf16_lossy(rest, u16::from_le_bytes));
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return (Encoding::Utf16Be { bom: true }, utf16_lossy(rest, u16::from_be_bytes));
    }
    // ASCII as UTF-16 is valid UTF-8 too, but full of NULs, so NULs send it to the UTF-16 check.
    let utf8 = std::str::from_utf8(bytes).ok();
    if let Some(text) = utf8.filter(|text| !text.contains('\0')) {
        return (Encoding::Utf8 { bom: false }, text.to_owned());
    }
    if let Some(found) = utf16_without_bom(bytes) {
        return found;
    }
    if let Some(text) = utf8 {
        return (Encoding::Utf8 { bom: false }, text.to_owned());
    }
    if let Some(multibyte) = multibyte_encoding_name(code_page).and_then(|name| encoding_rs::Encoding::for_label(name.as_bytes())) {
        return (Encoding::CodePage(code_page), multibyte.decode_without_bom_handling(bytes).0.into_owned());
    }
    let page = if SINGLE_BYTE_CODE_PAGES.contains(&code_page) { code_page } else { WESTERN };
    (Encoding::CodePage(page), bytes.iter().map(|&b| windows_byte(page, b)).collect())
}

fn utf16_lossy(bytes: &[u8], unit: fn([u8; 2]) -> u16) -> String {
    let (units, odd) = bytes.as_chunks::<2>();
    let mut text: String = char::decode_utf16(units.iter().map(|u| unit(*u))).map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER)).collect();
    if !odd.is_empty() {
        text.push(char::REPLACEMENT_CHARACTER);
    }
    text
}

/// UTF-16 without a mark: an even number of bytes, mostly Latin text, so at least a quarter of the
/// units have a zero high byte, which sits at odd offsets for little-endian and even ones for
/// big-endian. The bytes must decode with no unpaired surrogate and no NUL character.
// ponytail: text with few Latin characters (CJK, say) has too few zero bytes to detect without a
// mark; it falls through to the code page, and E07-S10's encoding override picks it.
fn utf16_without_bom(bytes: &[u8]) -> Option<(Encoding, String)> {
    let (units, odd) = bytes.as_chunks::<2>();
    if !odd.is_empty() || units.is_empty() {
        return None;
    }
    let high_zero_le = units.iter().filter(|[_, hi]| *hi == 0).count();
    let high_zero_be = units.iter().filter(|[hi, _]| *hi == 0).count();
    let (encoding, unit, zeros): (_, fn([u8; 2]) -> u16, _) = if high_zero_le > high_zero_be {
        (Encoding::Utf16Le { bom: false }, u16::from_le_bytes, high_zero_le)
    } else {
        (Encoding::Utf16Be { bom: false }, u16::from_be_bytes, high_zero_be)
    };
    if zeros * 4 < units.len() {
        return None;
    }
    let text: String = char::decode_utf16(units.iter().map(|u| unit(*u))).collect::<Result<_, _>>().ok()?;
    (!text.contains('\0')).then_some((encoding, text))
}

fn split(text: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let Some(at) = rest.find(['\r', '\n']) else {
            lines.push(Line { text: rest.to_owned(), ending: LineEnding::None });
            break;
        };
        let (line, tail) = rest.split_at(at);
        let (ending, tail) = if let Some(tail) = tail.strip_prefix("\r\n") {
            (LineEnding::CrLf, tail)
        } else if let Some(tail) = tail.strip_prefix('\r') {
            (LineEnding::Cr, tail)
        } else {
            (LineEnding::Lf, tail.get(1..).unwrap_or_default())
        };
        lines.push(Line { text: line.to_owned(), ending });
        rest = tail;
    }
    lines
}

fn endings(lines: &[Line]) -> LineEndings {
    let mut found = None;
    for line in lines {
        let style = match line.ending {
            LineEnding::None => continue,
            LineEnding::CrLf => LineEndings::Windows,
            LineEnding::Cr => LineEndings::Mac,
            LineEnding::Lf => LineEndings::Unix,
        };
        match found {
            None => found = Some(style),
            Some(seen) if seen != style => return LineEndings::Mixed,
            Some(_) => {}
        }
    }
    found.unwrap_or(LineEndings::None)
}

/// The system's legacy code page: Windows' ANSI code page, or [`WESTERN`] elsewhere, where the
/// system has none.
pub fn system_code_page() -> u16 {
    #[cfg(windows)]
    if let Some(page) = windows_code_page() {
        return page;
    }
    WESTERN
}

/// Windows' ANSI code page, the value GetACP returns, read from the registry with safe calls.
#[cfg(windows)]
fn windows_code_page() -> Option<u16> {
    let key = windows_registry::LOCAL_MACHINE.open(r"SYSTEM\CurrentControlSet\Control\Nls\CodePage").ok()?;
    key.get_string("ACP").ok()?.trim().parse().ok()
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn windows_names_its_code_page() {
        assert!(super::windows_code_page().is_some(), "a wrong key, value name or type would fall back to Windows-1252 unnoticed");
    }
}
