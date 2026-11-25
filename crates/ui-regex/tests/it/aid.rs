//! The aid driven by keyboard in a host window, read back through the accessibility tree.

use egui::accesskit::Role;
use egui::{Event, Key};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_regex::Regex;
use otd_ui_regex::{ELEMENTS, ELEMENTS_LABEL, EXAMPLES, EXAMPLES_LABEL, regex_aid};

/// A window with one labelled pattern field per entry, each hosting the aid.
struct Host {
    fields: Vec<String>,
}

impl Host {
    fn ui(&mut self, ui: &mut egui::Ui) {
        for (n, text) in self.fields.iter_mut().enumerate() {
            let label = ui.label(format!("Pattern {n}"));
            let field = ui.text_edit_singleline(text).labelled_by(label.id);
            regex_aid(ui, field.id, text);
        }
    }
}

fn harness(fields: &[&str]) -> Harness<'static, Host> {
    // ponytail: no default fonts in the build yet (see apps/otd's harness), so glyphs draw as boxes.
    let host = Host { fields: fields.iter().map(|&f| f.to_owned()).collect() };
    let mut harness = Harness::builder().allow_missing_glyphs().build_ui_state(|ui, host| host.ui(ui), host);
    harness.run();
    harness
}

fn press(harness: &mut Harness<'_, Host>, key: Key, times: usize) {
    for _ in 0..times {
        harness.key_press(key);
        harness.run();
    }
}

fn focused(harness: &Harness<'_, Host>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// Moves focus by Tab until the control named `name` has it, or fails after a screenful.
fn tab_to(harness: &mut Harness<'_, Host>, name: &str) {
    for _ in 0..20 {
        if focused(harness).is_some_and(|(_, label)| label.starts_with(name)) {
            return;
        }
        press(harness, Key::Tab, 1);
    }
    panic!("Tab never reached {name:?}");
}

/// Opens the menu whose button has focus and chooses the entry with `caption`, by arrow keys.
fn choose(harness: &mut Harness<'_, Host>, caption: &str) {
    press(harness, Key::Enter, 1);
    for _ in 0..40 {
        if focused(harness).is_some_and(|(_, label)| label.starts_with(caption)) {
            press(harness, Key::Enter, 1);
            return;
        }
        press(harness, Key::ArrowDown, 1);
    }
    panic!("the arrow keys never reached {caption:?}");
}

#[test]
fn an_element_goes_in_at_a_mid_text_cursor_and_typing_continues_after_it() {
    let mut harness = harness(&["héllo wörld"]);
    tab_to(&mut harness, "Pattern 0");
    // Before "wörld": five characters back from the end, past two-byte characters.
    press(&mut harness, Key::End, 1);
    press(&mut harness, Key::ArrowLeft, 5);
    tab_to(&mut harness, ELEMENTS_LABEL);
    choose(&mut harness, "Digit");

    assert_eq!(harness.state().fields[0], r"héllo \dwörld", "the element sits between the text either side of the cursor");
    assert_eq!(focused(&harness).map(|(role, _)| role), Some(Role::TextInput), "focus is back in the field");
    harness.event(Event::Text("!".to_owned()));
    harness.run();
    assert_eq!(harness.state().fields[0], r"héllo \d!wörld", "the cursor is right after the element");
}

#[test]
fn an_example_goes_into_the_field_whose_aid_chose_it() {
    let mut harness = harness(&["first", "second"]);
    // Past the first field's aid to the second field's.
    tab_to(&mut harness, "Pattern 1");
    tab_to(&mut harness, EXAMPLES_LABEL);
    choose(&mut harness, EXAMPLES[1].0);

    assert_eq!(harness.state().fields, ["first".to_owned(), format!("second{}", EXAMPLES[1].1)]);
}

#[test]
fn every_entry_the_aid_offers_is_a_valid_pattern() {
    for &(caption, example) in EXAMPLES {
        assert!(Regex::new(example).is_ok(), "example {caption:?}: {example:?}");
    }
    for &(caption, element) in ELEMENTS.iter().copied().flatten() {
        // Inside a pattern, after a group for `\1` to refer to and quantifiers to repeat.
        let pattern = format!("(a){element}b");
        assert!(Regex::new(&pattern).is_ok(), "element {caption:?} in {pattern:?}");
    }
}
