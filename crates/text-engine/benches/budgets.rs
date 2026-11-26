//! Checks the text engine's performance budgets (E03-S1-T6, recorded in the story's Decisions):
//!
//! - a 100,000-line pair with scattered changes shows its first rows within 1 s and completes
//!   within 10 s;
//! - a single-line edit in a 10,000-line pair recompares within 100 ms;
//! - the character differences of one changed row, up to the search's limits, come within 100 ms;
//! - comparing a 100,000-line pair grows the process by at most 64 MiB beyond its inputs.
//!
//! It exits with an error when any figure is over its budget. Memory is the process's resident
//! set, read with `ps` on macOS and Linux and `tasklist` (the working set) on Windows, in a
//! separate untimed pass; where neither can be read it says so and fails, rather than pass a
//! figure it never measured.
//!
//! Run it with `cargo bench -p otd-text-engine --bench budgets`. It is not part of `cargo test`,
//! because timings depend on how busy the machine is.

use std::ops::{ControlFlow, Range};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use otd_text_engine::{Line, LineEnding, Options, Pair, char_differences, compare_progressive};

const LARGE: usize = 100_000;
const FIRST_ROWS_WITHIN: Duration = Duration::from_secs(1);
const DONE_WITHIN: Duration = Duration::from_secs(10);

const EDITED: usize = 10_000;
const EDIT_WITHIN: Duration = Duration::from_millis(100);

const ROW_WITHIN: Duration = Duration::from_millis(100);

const MEMORY_WITHIN_KIB: u64 = 64 * 1024;

fn lines(texts: impl IntoIterator<Item = String>) -> Vec<Line> {
    texts.into_iter().map(|text| Line { text, ending: LineEnding::Lf }).collect()
}

/// `n` numbered lines, and a copy with a line changed, one deleted and one inserted every 1,000.
fn scattered(n: usize) -> (Vec<Line>, Vec<Line>) {
    let left = lines((0..n).map(|k| format!("line {k}")));
    let right = lines((0..n).filter(|k| k % 1_000 != 200).flat_map(|k| match k % 1_000 {
        100 => vec![format!("changed {k}")],
        300 => vec![format!("line {k}"), format!("inserted {k}")],
        _ => vec![format!("line {k}")],
    }));
    (left, right)
}

/// Source-like text where blank lines and closing braces dominate, so many points tie in the
/// alignment's search; the right side has a few scattered edits.
fn repetitive(n: usize) -> (Vec<Line>, Vec<Line>) {
    let shape = |k: usize| match k % 8 {
        0 | 4 => String::new(),
        3 | 7 => "}".to_owned(),
        6 => "    }".to_owned(),
        _ => format!("    call({});", k % 13),
    };
    let left = lines((0..n).map(shape));
    let right = lines((0..n).filter(|k| k % 997 != 5).map(|k| if k % 1_009 == 2 { "    changed();".to_owned() } else { shape(k) }));
    (left, right)
}

/// Times one comparison: when its first rows came and when it finished.
fn time_compare(left: &[Line], right: &[Line]) -> (Option<Duration>, Duration, usize) {
    let start = Instant::now();
    let mut first_rows = None;
    let done = compare_progressive(left, right, Options::default(), |batch| {
        if first_rows.is_none() && !batch.is_empty() {
            first_rows = Some(start.elapsed());
        }
        ControlFlow::<()>::Continue(())
    });
    let rows = if let ControlFlow::Continue(c) = &done { c.rows.len() } else { 0 };
    (first_rows, start.elapsed(), rows)
}

/// This process's resident memory in KiB, or `None` where it can't be read.
fn resident_kib() -> Option<u64> {
    let pid = std::process::id().to_string();
    if cfg!(windows) {
        // One CSV row of quoted fields whose last is the working set, such as "27,104 K"; the
        // thousands separator depends on the locale, so only that field's digits are kept.
        let out = Command::new("tasklist").args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"]).output().ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let digits: String = text.trim().rsplit("\",\"").next()?.chars().filter(char::is_ascii_digit).collect();
        digits.parse().ok()
    } else {
        let out = Command::new("ps").args(["-o", "rss=", "-p", &pid]).output().ok()?;
        String::from_utf8(out.stdout).ok()?.trim().parse().ok()
    }
}

/// How far a comparison grows the process above what it held before, sampled while it works and
/// again once it holds its result. `None` when memory can't be read.
fn growth_kib(left: &[Line], right: &[Line]) -> Option<u64> {
    let before = resident_kib()?;
    let mut peak = before;
    let mut polls = 0_usize;
    let done = compare_progressive(left, right, Options::default(), |_| {
        polls += 1;
        if polls.is_multiple_of(5_000) {
            peak = peak.max(resident_kib().unwrap_or(0));
        }
        ControlFlow::<()>::Continue(())
    });
    peak = peak.max(resident_kib()?);
    drop(done);
    Some(peak - before)
}

fn check(ok: &mut bool, what: &str, took: Duration, within: Duration) {
    let verdict = if took <= within { "ok" } else { "OVER BUDGET" };
    println!("  {what}: {took:?} (budget {within:?}) {verdict}");
    *ok &= took <= within;
}

fn main() -> ExitCode {
    let mut ok = true;

    // First, before other passes leave freed memory with the allocator and hide the growth.
    let (left, right) = scattered(LARGE);
    match growth_kib(&left, &right) {
        Some(kib) => {
            let verdict = if kib <= MEMORY_WITHIN_KIB { "ok" } else { "OVER BUDGET" };
            println!("{LARGE}-line pair, memory beyond the inputs: {kib} KiB (budget {MEMORY_WITHIN_KIB} KiB) {verdict}");
            ok &= kib <= MEMORY_WITHIN_KIB;
        }
        None => {
            println!("{LARGE}-line pair, memory: not measured, because the process's memory could not be read");
            ok = false;
        }
    }

    for (name, (left, right)) in [("scattered changes", scattered(LARGE)), ("repetitive text", repetitive(LARGE))] {
        let (first_rows, done, rows) = time_compare(&left, &right);
        println!("{LARGE}-line pair, {name}: {rows} rows");
        check(&mut ok, "first rows", first_rows.unwrap_or(Duration::MAX), FIRST_ROWS_WITHIN);
        check(&mut ok, "complete", done, DONE_WITHIN);
    }

    // A single-line edit at the top, middle and end of a 10,000-line pair, on either side: replace,
    // insert and delete. The slowest counts.
    let (left, right) = scattered(EDITED);
    let edit_line = || vec![Line { text: "edited".to_owned(), ending: LineEnding::Lf }];
    let mut pair = Pair::new(left, right);
    let mut slowest = Duration::ZERO;
    for at in [0, EDITED / 2, EDITED - 2] {
        let edits: [(bool, Range<usize>, Vec<Line>); 6] = [
            (true, at..at + 1, edit_line()),
            (false, at..at + 1, edit_line()),
            (true, at..at, edit_line()),
            (false, at..at, edit_line()),
            (true, at..at + 1, Vec::new()),
            (false, at..at + 1, Vec::new()),
        ];
        for (on_left, range, with) in edits {
            let start = Instant::now();
            let edited = if on_left { pair.edit_left(range, with) } else { pair.edit_right(range, with) };
            slowest = slowest.max(start.elapsed());
            if edited.is_err() {
                println!("  an edit fell outside the text");
                ok = false;
            }
        }
    }
    println!("{EDITED}-line pair, single-line edits:");
    check(&mut ok, "slowest recompare", slowest, EDIT_WITHIN);

    // The slowest rows for the character search: lines that differ by just under its edit limit,
    // and long lines that run it out of steps.
    let near_edit_limit = (
        (0..3_000).map(|k| char::from(b'a' + (k % 26) as u8)).collect::<String>(),
        (0..3_000).map(|k| if k % 3 == 0 { 'X' } else { char::from(b'a' + (k % 26) as u8) }).collect::<String>(),
    );
    // A periodic 200,000-character line with every 200th character changed: long runs of equal
    // characters on many diagonals, so the search spends its steps before its edits.
    let periodic = "ab".repeat(100_000);
    let out_of_steps = (periodic.clone(), periodic.char_indices().map(|(k, c)| if k % 200 == 0 { 'X' } else { c }).collect::<String>());
    println!("character differences in one changed row:");
    for (name, (left, right)) in [("near the edit limit", near_edit_limit), ("past the step limit", out_of_steps)] {
        let start = Instant::now();
        let result = char_differences(&left, &right);
        check(&mut ok, &format!("{name} (exact: {})", result.exact), start.elapsed(), ROW_WITHIN);
    }

    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
