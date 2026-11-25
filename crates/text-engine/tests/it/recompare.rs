//! Recompare after edits: random edit sequences on the alignment corpus, checked against a full
//! comparison of the edited text after every edit.

use otd_text_engine::{Line, LineEnding, Pair, compare};

use crate::alignment::{CORPUS, side};

/// A fixed-seed xorshift generator, so every run makes the same edits.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        usize::try_from(self.0 % n as u64).unwrap()
    }
}

/// Edit sequences per corpus case: `PROPTEST_CASES`, the suite's shared case budget, or 8.
fn sequences() -> usize {
    std::env::var("PROPTEST_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(8)
}

#[test]
fn after_every_edit_the_rows_equal_a_full_comparison_of_the_edited_text() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for case in CORPUS {
        // Replacement lines reuse the case's own texts, so edits make and break matches and
        // repeats; a new text and a changed line ending stand for a fresh edit.
        let mut pool: Vec<Line> = side(case.left).into_iter().chain(side(case.right)).collect();
        pool.push(Line { text: "new".to_owned(), ending: LineEnding::Lf });
        pool.push(Line { text: "a".to_owned(), ending: LineEnding::CrLf });
        for sequence in 0..sequences() {
            let mut pair = Pair::new(side(case.left), side(case.right));
            let (mut left, mut right) = (side(case.left), side(case.right));
            for step in 0..12 {
                let on_left = rng.below(2) == 0;
                let model = if on_left { &mut left } else { &mut right };
                let start = rng.below(model.len() + 1);
                let end = start + rng.below((model.len() - start).min(3) + 1);
                let with: Vec<Line> = (0..rng.below(4)).map(|_| pool[rng.below(pool.len())].clone()).collect();
                *model = [&model[..start], &with[..], &model[end..]].concat();

                let got = if on_left { pair.edit_left(start..end, with) } else { pair.edit_right(start..end, with) }.cloned();
                let at = format!("{}: sequence {sequence}, step {step}", case.name);
                assert_eq!((pair.left(), pair.right()), (&left[..], &right[..]), "{at}: the edit landed elsewhere");
                assert_eq!(got.as_ref(), Ok(&compare(&left, &right)), "{at}");
            }
        }
    }
}
