//! Standard line alignment and sections, checked on a corpus of before/after pairs.
//!
//! Rows are written as pictures: `a` is a same row, `b ~ x` a changed row (left ~ right), `b <` a
//! left line against a gap and `> x` a right line against a gap.

use otd_text_engine::{Comparison, Line, LineEnding, Options, RowKind, compare, compare_with};

pub(crate) fn side(texts: &[&str]) -> Vec<Line> {
    texts.iter().map(|t| Line { text: (*t).to_owned(), ending: LineEnding::Lf }).collect()
}

fn text(lines: &[Line], at: Option<usize>) -> &str {
    at.and_then(|k| lines.get(k)).map_or("?", |l| l.text.as_str())
}

fn picture(c: &Comparison, left: &[Line], right: &[Line]) -> Vec<String> {
    c.rows
        .iter()
        .map(|row| match row.kind {
            RowKind::Same => text(left, row.left).to_owned(),
            RowKind::Changed => format!("{} ~ {}", text(left, row.left), text(right, row.right)),
            RowKind::LeftOnly => format!("{} <", text(left, row.left)),
            RowKind::RightOnly => format!("> {}", text(right, row.right)),
        })
        .collect()
}

/// The picture of the same comparison with the sides swapped.
fn mirror(rows: &[&str]) -> Vec<String> {
    rows.iter()
        .map(|row| {
            if let Some((l, r)) = row.split_once(" ~ ") {
                format!("{r} ~ {l}")
            } else if let Some(l) = row.strip_suffix(" <") {
                format!("> {l}")
            } else if let Some(r) = row.strip_prefix("> ") {
                format!("{r} <")
            } else {
                (*row).to_owned()
            }
        })
        .collect()
}

pub(crate) struct Case {
    pub(crate) name: &'static str,
    pub(crate) left: &'static [&'static str],
    pub(crate) right: &'static [&'static str],
    expect: Expect,
    sections: usize,
}

enum Expect {
    /// The exact rows, where every reasonable alignment agrees on them.
    Rows(&'static [&'static str]),
    /// Only how many lines pair as same, where more than one placement is as good.
    Same(usize),
}

pub(crate) const CORPUS: &[Case] = &[
    Case { name: "identical", left: &["a", "b", "c"], right: &["a", "b", "c"], expect: Expect::Rows(&["a", "b", "c"]), sections: 0 },
    Case { name: "both empty", left: &[], right: &[], expect: Expect::Rows(&[]), sections: 0 },
    Case { name: "one side empty", left: &[], right: &["a", "b"], expect: Expect::Rows(&["> a", "> b"]), sections: 1 },
    Case { name: "a changed line", left: &["a", "b", "c"], right: &["a", "x", "c"], expect: Expect::Rows(&["a", "b ~ x", "c"]), sections: 1 },
    Case {
        name: "an insertion in the middle",
        left: &["a", "b", "c"],
        right: &["a", "b", "n1", "n2", "c"],
        expect: Expect::Rows(&["a", "b", "> n1", "> n2", "c"]),
        sections: 1,
    },
    Case { name: "an insertion at the top", left: &["a", "b"], right: &["n", "a", "b"], expect: Expect::Rows(&["> n", "a", "b"]), sections: 1 },
    Case { name: "a deletion at the end", left: &["a", "b", "c"], right: &["a", "b"], expect: Expect::Rows(&["a", "b", "c <"]), sections: 1 },
    Case {
        name: "a change next to an insertion pairs from the top",
        left: &["a", "b", "c", "d"],
        right: &["a", "x", "y", "c", "d"],
        expect: Expect::Rows(&["a", "b ~ x", "> y", "c", "d"]),
        sections: 1,
    },
    Case {
        name: "two separate changes",
        left: &["a", "b", "c", "d", "e", "f", "g"],
        right: &["a", "x", "c", "d", "e", "y", "g"],
        expect: Expect::Rows(&["a", "b ~ x", "c", "d", "e", "f ~ y", "g"]),
        sections: 2,
    },
    Case {
        name: "a small block moved past a larger one",
        left: &["m1", "m2", "a", "b", "c", "d", "e", "f"],
        right: &["a", "b", "c", "d", "e", "f", "m1", "m2"],
        expect: Expect::Rows(&["m1 <", "m2 <", "a", "b", "c", "d", "e", "f", "> m1", "> m2"]),
        sections: 2,
    },
    Case {
        name: "a repeated line ahead of its block doesn't pull the block apart",
        left: &["c", "x", "y", "a", "b", "c", "d"],
        right: &["a", "b", "c", "d"],
        expect: Expect::Rows(&["c <", "x <", "y <", "a", "b", "c", "d"]),
        sections: 1,
    },
    Case {
        name: "repeated blank lines pair between changes",
        left: &["h", "a1", "", "a2", "", "a3", "t1", "t2", "t3"],
        right: &["h", "b1", "b1x", "", "b2", "", "b3", "t1", "t2", "t3"],
        expect: Expect::Rows(&["h", "a1 ~ b1", "> b1x", "", "a2 ~ b2", "", "a3 ~ b3", "t1", "t2", "t3"]),
        sections: 3,
    },
    Case {
        name: "a line deleted among alternating lines",
        left: &["h", "y", "x", "y", "x", "y", "t"],
        right: &["h", "x", "y", "x", "y", "t"],
        expect: Expect::Rows(&["h", "y <", "x", "y", "x", "y", "t"]),
        sections: 1,
    },
    Case {
        name: "a deletion next to a change",
        left: &["h1", "h2", "h3", "}", "b", "}", "c", "t1", "t2", "t3"],
        right: &["h1", "h2", "h3", "b", "}", "x", "t1", "t2", "t3"],
        expect: Expect::Rows(&["h1", "h2", "h3", "} <", "b", "}", "c ~ x", "t1", "t2", "t3"]),
        sections: 2,
    },
    Case {
        // Either copy can be the one against a gap; only the shape is fixed.
        name: "a repeated line removed",
        left: &["x", "x", "x"],
        right: &["x", "x"],
        expect: Expect::Same(2),
        sections: 1,
    },
    Case {
        // The new function can be cut at either blank line; only the shape is fixed.
        name: "a function inserted between functions",
        left: &["fn a() {", "x", "}", "", "fn b() {", "y", "}"],
        right: &["fn a() {", "x", "}", "", "fn n() {", "z", "}", "", "fn b() {", "y", "}"],
        expect: Expect::Same(7),
        sections: 1,
    },
];

/// Each corpus case, then the same case with its sides swapped.
fn both_ways() -> impl Iterator<Item = (String, Vec<Line>, Vec<Line>, &'static Case)> {
    CORPUS.iter().flat_map(|case| {
        [
            (case.name.to_owned(), side(case.left), side(case.right), case),
            (format!("{} (swapped)", case.name), side(case.right), side(case.left), case),
        ]
    })
}

#[test]
fn the_corpus_gives_the_expected_rows_and_section_counts() {
    for (name, left, right, case) in both_ways() {
        let c = compare(&left, &right);
        let got = picture(&c, &left, &right);
        match case.expect {
            Expect::Rows(rows) if name.ends_with("(swapped)") => assert_eq!(got, mirror(rows), "{name}"),
            Expect::Rows(rows) => assert_eq!(got, rows, "{name}"),
            Expect::Same(n) => assert_eq!(c.rows.iter().filter(|r| r.kind == RowKind::Same).count(), n, "{name}: {got:?}"),
        }
        assert_eq!(c.sections.len(), case.sections, "{name}: {got:?}");
    }
}

#[test]
fn every_line_appears_once_in_input_order_on_its_side() {
    for (name, left, right, ..) in both_ways() {
        let c = compare(&left, &right);
        let lefts: Vec<usize> = c.rows.iter().filter_map(|r| r.left).collect();
        let rights: Vec<usize> = c.rows.iter().filter_map(|r| r.right).collect();
        assert_eq!(lefts, (0..left.len()).collect::<Vec<_>>(), "{name}");
        assert_eq!(rights, (0..right.len()).collect::<Vec<_>>(), "{name}");
    }
}

#[test]
fn a_row_kind_says_what_the_row_holds() {
    for (name, left, right, ..) in both_ways() {
        for row in compare(&left, &right).rows {
            let (l, r) = (row.left.and_then(|k| left.get(k)), row.right.and_then(|k| right.get(k)));
            let ok = match row.kind {
                RowKind::Same => l.is_some() && l == r,
                RowKind::Changed => l.is_some() && r.is_some() && l != r,
                RowKind::LeftOnly => l.is_some() && row.right.is_none(),
                RowKind::RightOnly => r.is_some() && row.left.is_none(),
            };
            assert!(ok, "{name}: {row:?}");
        }
    }
}

#[test]
fn sections_are_the_maximal_runs_of_differing_rows_and_repeat_exactly() {
    for (name, left, right, ..) in both_ways() {
        let c = compare(&left, &right);
        let same = |k: usize| c.rows.get(k).is_some_and(|r| r.kind == RowKind::Same);
        let mut in_section = vec![false; c.rows.len()];
        let mut previous_end = None;
        for s in &c.sections {
            assert!(!s.is_empty() && s.clone().all(|k| !same(k)), "{name}: {s:?} holds a same row");
            assert!(s.start == 0 || same(s.start - 1), "{name}: {s:?} starts mid-run");
            assert!(s.end == c.rows.len() || same(s.end), "{name}: {s:?} stops mid-run");
            assert!(previous_end.is_none_or(|e| e < s.start), "{name}: sections out of order or touching");
            previous_end = Some(s.end);
            s.clone().for_each(|k| in_section[k] = true);
        }
        assert!((0..c.rows.len()).all(|k| same(k) || in_section[k]), "{name}: a differing row is in no section");
        assert_eq!(compare(&left, &right), c, "{name}: a second run differs");
    }
}

#[test]
fn lines_align_on_their_text_and_with_line_endings_compared_a_different_ending_makes_the_row_changed() {
    let crlf = |t: &str| Line { text: t.to_owned(), ending: LineEnding::CrLf };
    let left: Vec<Line> = ["a", "b", "c", "d"].map(crlf).into();
    let right = side(&["a", "b", "n", "c", "d"]);
    let c = compare_with(&left, &right, Options { line_endings: true });
    let kinds: Vec<RowKind> = c.rows.iter().map(|r| r.kind).collect();
    use RowKind::{Changed, RightOnly};
    assert_eq!(kinds, [Changed, Changed, RightOnly, Changed, Changed]);
    assert_eq!(c.rows.get(2).and_then(|r| r.right), Some(2), "the inserted line is the one against the gap");
}

#[test]
fn one_deleted_line_among_repeated_lines_is_one_gap_either_way_round() {
    let mut texts: Vec<Vec<&str>> = vec![vec![]];
    for len in 1..=8 {
        let shorter: Vec<Vec<&str>> = texts.iter().filter(|t| t.len() == len - 1).cloned().collect();
        texts.extend(shorter.into_iter().flat_map(|t| ["x", "y"].map(|s| [t.as_slice(), &[s]].concat())));
    }
    for full in &texts {
        for k in 0..full.len() {
            let mut cut = full.clone();
            cut.remove(k);
            for (left, right) in [(side(full), side(&cut)), (side(&cut), side(full))] {
                let c = compare(&left, &right);
                let same = c.rows.iter().filter(|r| r.kind == RowKind::Same).count();
                assert!(same == cut.len() && c.sections.len() == 1, "{full:?} without line {k}: {:?}", picture(&c, &left, &right));
            }
        }
    }
}
