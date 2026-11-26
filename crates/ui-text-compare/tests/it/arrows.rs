//! The toolbar's section arrows (E07-S2-T4), driven by keyboard and clicks through the real shell
//! and read back through the accessibility tree and the frame's painted shapes.

use egui::accesskit::Role;
use egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_ui::colour::{self, CHROME, ColourRoles, Palette};
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::arrows::{ARROW, ROLES};
use otd_ui_text_compare::{LEFT, NEXT_SECTION, PREVIOUS_SECTION, SESSION_TYPE, TextCompareView, commands};

fn harness(folder: &tempfile::TempDir, left: &str, right: &str) -> Harness<'static, Shell> {
    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = [("left.txt", left), ("right.txt", right)]
            .map(|(name, text)| {
                let path = folder.path().join(name);
                std::fs::write(&path, text).unwrap();
                path.display().to_string()
            })
            .to_vec();
    }
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().with_size(vec2(900.0, 600.0)).build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    // As the app does, the chrome's and the arrows' roles.
    colour::install(&harness.ctx, ColourRoles::new(CHROME.iter().chain(&ROLES).copied()).expect("distinct ids"));
    harness.run();
    harness
}

/// How many line segments the last frame painted in the arrow role's value for `palette`.
fn yellow_strokes(harness: &Harness<'_, Shell>, palette: Palette) -> usize {
    let rgba = ARROW.value(palette);
    let yellow = egui::Color32::from_rgba_unmultiplied(rgba.r, rgba.g, rgba.b, rgba.a);
    harness
        .output()
        .shapes
        .iter()
        .filter(|clipped| matches!(&clipped.shape, egui::Shape::LineSegment { stroke, .. } if stroke.color == yellow))
        .count()
}

fn key(harness: &mut Harness<'_, Shell>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

/// Moves focus with Tab to the left editor.
fn focus_left(harness: &mut Harness<'_, Shell>) {
    for _ in 0..40 {
        if harness.query_by(|node| node.is_focused()).is_some_and(|node| node.accesskit_node().role() == Role::MultilineTextInput) {
            break;
        }
        key(harness, Modifiers::NONE, Key::Tab);
    }
    assert_eq!(harness.get_by(|node| node.is_focused()).accesskit_node().label().as_deref(), Some(LEFT), "the left editor has focus");
}

/// Whether the toolbar's Previous and Next arrows are enabled.
fn arrows(harness: &Harness<'_, Shell>) -> (bool, bool) {
    let enabled = |name: &str| {
        let button = harness.get_by(|node| {
            node.role() == Role::Button && node.label().as_deref() == Some(name) && node.parent().is_some_and(|parent| parent.role() == Role::Toolbar)
        });
        !button.accesskit_node().is_disabled()
    };
    (enabled(PREVIOUS_SECTION), enabled(NEXT_SECTION))
}

fn click(harness: &mut Harness<'_, Shell>, name: &str) {
    harness.get_by(|node| node.role() == Role::Button && node.label().as_deref() == Some(name)).click();
    harness.run();
}

/// The chosen pane and its caret's line.
fn caret(harness: &Harness<'_, Shell>) -> (usize, usize) {
    let view = harness.state().view::<TextCompareView>().unwrap();
    let side = view.chosen();
    (side, view.pane(side).unwrap().selection.head.line)
}

#[test]
fn at_the_last_section_the_next_arrow_turns_off_and_the_previous_one_stays_on() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "a\nB\nc\nD\ne\n", "a\nb\nc\nd\ne\n");
    assert_eq!(arrows(&harness), (false, true), "above both sections only Next is on");

    click(&mut harness, NEXT_SECTION);
    assert_eq!(caret(&harness), (0, 1), "Next lands on the first section");
    assert_eq!(arrows(&harness), (false, true), "from inside the first section nothing lies above");

    click(&mut harness, NEXT_SECTION);
    assert_eq!(caret(&harness), (0, 3), "then on the last one");
    assert_eq!(arrows(&harness), (true, false), "at the last section Next is off and Previous on");

    assert!(harness.state_mut().run("text-compare.previous_section"), "the arrow's command runs without the window");
    harness.run();
    assert_eq!(caret(&harness), (0, 1), "Previous goes back to the first section");
}

#[test]
fn an_edit_that_removes_every_difference_turns_both_arrows_off() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "a\nbx\nc\n", "a\nb\nc\n");
    focus_left(&mut harness);
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    assert_eq!(arrows(&harness), (false, false), "the caret is on the only section");
    key(&mut harness, Modifiers::NONE, Key::ArrowUp);
    assert_eq!(arrows(&harness), (false, true), "the arrows follow the caret");

    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    key(&mut harness, Modifiers::NONE, Key::End);
    key(&mut harness, Modifiers::NONE, Key::Backspace);
    key(&mut harness, Modifiers::NONE, Key::ArrowUp);
    assert_eq!(arrows(&harness), (false, false), "with the files the same, neither arrow is on");
}

#[test]
fn a_section_of_lines_only_the_other_file_has_counts_and_previous_lands_in_the_other_pane() {
    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "a\nb\n", "a\nx\ny\nb\n");
    focus_left(&mut harness);
    key(&mut harness, Modifiers::NONE, Key::ArrowDown);
    assert_eq!(caret(&harness), (0, 1), "the left caret is on b, just below the right side's insertion");
    assert_eq!(arrows(&harness), (true, false), "the insertion above lights Previous");

    click(&mut harness, PREVIOUS_SECTION);
    assert_eq!(caret(&harness), (1, 1), "Previous goes to the inserted lines, in the right pane");
}

#[test]
fn an_arrow_is_drawn_in_the_arrow_colour_only_while_it_is_on() {
    let same = tempfile::tempdir().unwrap();
    let unchanged = harness(&same, "a\nb\n", "a\nb\n");
    assert_eq!(arrows(&unchanged), (false, false));
    assert_eq!(yellow_strokes(&unchanged, Palette::Light), 0, "a disabled arrow is greyed, not yellow");

    let folder = tempfile::tempdir().unwrap();
    let mut harness = harness(&folder, "a\nB\nc\nD\ne\n", "a\nb\nc\nd\ne\n");
    for palette in [Palette::Light, Palette::Dark] {
        colour::set_palette(&harness.ctx, palette);
        harness.run();
        assert_eq!(arrows(&harness), (false, true));
        assert!(yellow_strokes(&harness, palette) > 0, "the enabled Next arrow is drawn in the arrow colour in {palette:?}");
    }
}
