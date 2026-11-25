//! Times progressive results and cancellation on 200,000-line pairs (E03-S1-T4): how soon the
//! first rows arrive, how long the comparison takes, and the longest gap between two calls of the
//! caller's step. A cancel takes effect at the next call, so that gap bounds how long a cancel
//! waits. Exits with an error when a gap exceeds half a second.
//!
//! Run it with `cargo bench -p otd-text-engine --bench progress`. It is not part of `cargo test`,
//! because timings depend on how busy the machine is.

use std::ops::ControlFlow;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use otd_text_engine::{Line, LineEnding, compare_progressive};

const LINES: usize = 200_000;
const CANCEL_WITHIN: Duration = Duration::from_millis(500);

fn lines(texts: impl IntoIterator<Item = String>) -> Vec<Line> {
    texts.into_iter().map(|text| Line { text, ending: LineEnding::Lf }).collect()
}

fn main() -> ExitCode {
    let numbered = lines((0..LINES).map(|k| format!("line {k}")));
    let scattered = lines((0..LINES).filter(|k| k % 1_000 != 200).flat_map(|k| match k % 1_000 {
        100 => vec![format!("changed {k}")],
        300 => vec![format!("line {k}"), format!("inserted {k}")],
        _ => vec![format!("line {k}")],
    }));
    let other = lines((0..LINES).map(|k| format!("other {k}")));
    let mut moved = numbered.clone();
    moved.rotate_left(LINES / 2);

    let mut ok = true;
    for (name, left, right) in
        [("scattered changes", &numbered, &scattered), ("nothing in common", &numbered, &other), ("halves swapped", &numbered, &moved)]
    {
        let start = Instant::now();
        let (mut last, mut gap, mut first_rows) = (start, Duration::ZERO, None);
        let done = compare_progressive(left, right, |batch| {
            let now = Instant::now();
            gap = gap.max(now - last);
            last = now;
            if first_rows.is_none() && !batch.is_empty() {
                first_rows = Some(now - start);
            }
            ControlFlow::<()>::Continue(())
        });
        let total = start.elapsed();
        // The work after the last poll (the sections pass) counts too.
        gap = gap.max(last.elapsed());
        let rows = if let ControlFlow::Continue(c) = &done { c.rows.len() } else { 0 };
        println!("{name}: {rows} rows; first rows after {first_rows:?}, done after {total:?}, longest gap between polls {gap:?}");
        if gap > CANCEL_WITHIN {
            println!("  a cancel could wait longer than {CANCEL_WITHIN:?}");
            ok = false;
        }
    }
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
