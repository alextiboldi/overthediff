//! The hostile-input sweep: craft-testkit's `hammer` runs a parser on a valid seed, every truncation
//! of it and damaged copies (bit flips, extreme values, deletions), and fails on a panic or a hang.
//! Every reader of files, settings or network replies gets a sweep of this shape.
//!
//! ponytail: no reader exists yet, so this sweeps a sample record parser defined here. The first real
//! reader gets its own sweep in its crate's `tests/it/` and this sample goes.

use craft_testkit::hostile::{HammerConfig, hammer};

/// A sample format: the magic `OTD1`, a `u16` record count, then per record a `u8` key length, the
/// key's UTF-8 bytes, a `u32` value length and the value's bytes. Integers are little-endian.
fn parse(data: &[u8]) -> Option<Vec<(String, Vec<u8>)>> {
    let mut rest = data.strip_prefix(b"OTD1")?;
    let count = u16::from_le_bytes(take(&mut rest, 2)?.try_into().ok()?);
    // The count is untrusted, so it doesn't size the allocation.
    let mut records = Vec::new();
    for _ in 0..count {
        let key_len = usize::from(*take(&mut rest, 1)?.first()?);
        let key = std::str::from_utf8(take(&mut rest, key_len)?).ok()?.to_owned();
        let value_len = usize::try_from(u32::from_le_bytes(take(&mut rest, 4)?.try_into().ok()?)).ok()?;
        records.push((key, take(&mut rest, value_len)?.to_vec()));
    }
    rest.is_empty().then_some(records)
}

/// Splits `n` bytes off the front of `rest`, or `None` when fewer remain.
fn take<'a>(rest: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
    let (head, tail) = rest.split_at_checked(n)?;
    *rest = tail;
    Some(head)
}

fn record(key: &str, value: &[u8]) -> Vec<u8> {
    let mut out = vec![u8::try_from(key.len()).unwrap_or(u8::MAX)];
    out.extend_from_slice(key.as_bytes());
    out.extend_from_slice(&u32::try_from(value.len()).unwrap_or(u32::MAX).to_le_bytes());
    out.extend_from_slice(value);
    out
}

#[test]
fn the_sample_parser_never_panics_or_hangs_on_truncated_or_damaged_input() {
    let mut seed = b"OTD1".to_vec();
    seed.extend_from_slice(&2u16.to_le_bytes());
    seed.extend(record("left", b"a.txt"));
    seed.extend(record("right", b"b.txt"));
    // A sweep over a parser that rejects its own seed would only exercise the first check.
    assert!(parse(&seed).is_some_and(|records| records.len() == 2), "the seed parses");

    hammer(&HammerConfig::new("sample records", 2_000), &seed, |data| {
        let _ = parse(data);
    });
}
