//! The keyboard-and-accessibility UI harness: drive a window by keyboard with egui_kittest and,
//! after each step, read from the accessibility tree where focus is, and the focused node's role
//! and name. Every UI test copies this shape.
//!
//! ponytail: E01-S1's command registry, translation lookup and colour roles don't exist yet, so this
//! drives a small window and command of its own. Once the shell exists, UI tests drive it instead.

use eframe::egui::accesskit::Role;
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

const COMPARE: &str = "Compare";
const FIND_WHAT: &str = "Find what";
const MATCH_CASE: &str = "Match case";
/// The test command's shortcut. `COMMAND` is Cmd on macOS and Ctrl elsewhere.
const SHOW_FIND_BAR: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::F);

/// A window with one button and a find bar that the test command shows. The find bar takes focus
/// when it opens, and Esc closes it and gives focus back to where it was.
#[derive(Default)]
struct TestWindow {
    find_bar_open: bool,
    /// The widget that had focus when the find bar opened.
    opener: Option<egui::Id>,
    query: String,
    match_case: bool,
}

impl TestWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let opened = !self.find_bar_open && ui.input_mut(|i| i.consume_shortcut(&SHOW_FIND_BAR));
        if opened {
            self.find_bar_open = true;
            self.opener = ui.memory(|m| m.focused());
        }
        if self.find_bar_open && ui.input(|i| i.key_pressed(Key::Escape)) {
            self.find_bar_open = false;
            if let Some(opener) = self.opener.take() {
                ui.memory_mut(|m| m.request_focus(opener));
            }
        }

        let _ = ui.button(COMPARE);
        if self.find_bar_open {
            ui.horizontal(|ui| {
                let label = ui.label(FIND_WHAT);
                let field = ui.text_edit_singleline(&mut self.query).labelled_by(label.id);
                if opened {
                    field.request_focus();
                }
                ui.checkbox(&mut self.match_case, MATCH_CASE);
            });
        }
    }
}

/// The role and name of the node that has keyboard focus, or `None` when nothing has it.
fn focused(harness: &Harness<'_, TestWindow>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

fn harness() -> Harness<'static, TestWindow> {
    // ponytail: the app is built without egui's default fonts yet, so text draws as boxes. Drop
    // `allow_missing_glyphs` when the first task that draws text enables them.
    Harness::builder().allow_missing_glyphs().build_ui_state(|ui, window| window.ui(ui), TestWindow::default())
}

#[test]
fn the_find_bar_shortcut_moves_focus_into_it_and_esc_gives_focus_back() {
    let mut harness = harness();
    harness.run();
    assert_eq!(focused(&harness), None, "nothing has focus before the first key");

    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(focused(&harness), Some((Role::Button, COMPARE.to_owned())), "Tab focuses the first control");

    harness.key_press_modifiers(SHOW_FIND_BAR.modifiers, SHOW_FIND_BAR.logical_key);
    harness.run();
    assert_eq!(focused(&harness), Some((Role::TextInput, FIND_WHAT.to_owned())), "the shortcut focuses the find bar's field");

    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(focused(&harness), Some((Role::CheckBox, MATCH_CASE.to_owned())), "Tab moves on inside the find bar");

    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.query_by_label(MATCH_CASE).is_none(), "Esc closes the find bar");
    assert_eq!(focused(&harness), Some((Role::Button, COMPARE.to_owned())), "Esc gives focus back to where it was");
}
