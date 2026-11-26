//! The Text Editing page's *Number of context lines* and *Show filtered line counts* (E07-S3-T3):
//! applied from the Options dialog by keyboard, and read back from an open view through the
//! accessibility tree.

use std::collections::BTreeMap;

use egui::accesskit::Role;
use egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_text_compare::Filter;
use otd_ui::{APPLY, Shell, Value, View as _};
use otd_ui_text_compare::filtered::{CONTEXT_LINES, CONTEXT_LINES_ID, options};
use otd_ui_text_compare::{SESSION_TYPE, TextCompareView, commands};

/// The shell with the two options, *N* = 2 applied, and a view under Show Context comparing files
/// that differ in their first and last lines, with 41 same lines between.
fn harness(folder: &tempfile::TempDir) -> Harness<'static, Shell> {
    let body: String = (1..=41).map(|n| format!("line {n}\n")).collect();
    let mut view = TextCompareView::new();
    view.set_filter(Filter::Context);
    if let Some(session) = view.session() {
        session.specs = [("left.txt", format!("left\n{body}left end\n")), ("right.txt", format!("right\n{body}right end\n"))]
            .map(|(name, text)| {
                let path = folder.path().join(name);
                std::fs::write(&path, text).unwrap();
                path.display().to_string()
            })
            .to_vec();
    }
    let saved = BTreeMap::from([(CONTEXT_LINES_ID.to_owned(), Value::Number(2.0))]);
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands()).with_options(options(), saved).unwrap();
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 1200.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

/// How many panes show a summary saying `n` lines are hidden.
fn summaries(harness: &Harness<'_, Shell>, n: usize) -> usize {
    harness.query_all_by_label(&format!("{n} lines hidden")).count()
}

fn press(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

fn tab_to(harness: &mut Harness<'_, Shell>, role: Role, name: &str) {
    for _ in 0..30 {
        let focused = harness.query_by(|node| node.is_focused()).map(|node| node.accesskit_node());
        if focused.is_some_and(|node| node.role() == role && node.label().as_deref() == Some(name)) {
            return;
        }
        press(harness, Modifiers::NONE, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

#[test]
fn a_hidden_run_says_how_many_lines_it_hides_and_follows_an_applied_context_count() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder);
    // Two lines either side of each different line leave lines 3 to 39 hidden: 37 of them.
    assert_eq!(summaries(&harness, 37), 2, "both panes summarise the 37 hidden lines");
    assert!(harness.query_by_label("line 20").is_none(), "the hidden lines aren't drawn");

    assert!(harness.state_mut().run("otd.options"));
    harness.run();
    tab_to(&mut harness, Role::SpinButton, CONTEXT_LINES);
    press(&mut harness, Modifiers::COMMAND, Key::A);
    harness.event(egui::Event::Text("3".to_owned()));
    harness.run();
    tab_to(&mut harness, Role::Button, APPLY);
    press(&mut harness, Modifiers::NONE, Key::Enter);

    assert_eq!(summaries(&harness, 37), 0, "the old count is gone");
    assert_eq!(summaries(&harness, 35), 2, "one more line either side shows, so 35 are hidden");
}
