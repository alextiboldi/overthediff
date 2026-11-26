//! The shell's text goes through the translation lookup (E01-S1-T13): a caption with a test
//! translation shows translated, one without shows its English key, and in the pseudo locale,
//! which marks every looked-up string `⟦…⟧`, every text in the window and in the Session Settings
//! and Options dialogs is one marked string.

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::{Embedded, I18n, Lang, embedded, set_language, t};
use otd_ui::command::Command;
use otd_ui::{
    APPLY, CANCEL, CANT_SAVE, CHOOSE_A_PAGE, CLOSE_TAB, DESCRIPTION, DONT_SAVE, Field, HOME, INPUTS, INSERT, LOAD_TIME, NEW_TAB, NO_SESSION_TYPES,
    OK, OPTION_PAGES, OPTIONS, OptionSpec, SAVE, SAVE_CHANGES, SCOPE, SESSION_SETTINGS, Session, SessionType, Setting, Shell, Status, THIS_VIEW_ONLY,
    UNSAVED_LOST, Value, View,
};

const TRANSLATED: &str = "Alpha Compare";
const UNTRANSLATED: &str = "Beta Compare";
const SIDE: &str = "Left";
const TAB: &str = "Comparison";
const LABEL: &str = "Ignore case";
const CHOICE_LABEL: &str = "Unit";
const CHOICE: &str = "Lines";
const PAGE: &str = "Text";
const SUBPAGE: &str = "Editing";
const OPTION: &str = "Wrap lines";

/// A test view whose own text, like a real view's, is looked up.
struct HarnessView(Session);

impl View for HarnessView {
    fn title(&self) -> String {
        t("Harness view").to_owned()
    }

    fn status(&self) -> Status {
        let field = |id, label: &str, value: &str| Field { id, label: t(label).to_owned(), value: t(value).to_owned() };
        Status {
            result: Some(field("result", "Result", "Identical")),
            fields: vec![field("own", "Own", "Value")],
            insert: Some(true),
            load_time: Some(Duration::from_millis(1500)),
        }
    }

    fn session(&mut self) -> Option<&mut Session> {
        Some(&mut self.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(t("Harness contents"));
    }
}

/// A test view with changes it can't save, so closing it asks and Save shows why it didn't.
struct UnsavedView;

impl View for UnsavedView {
    fn title(&self) -> String {
        t("Harness view").to_owned()
    }

    fn unsaved(&self) -> bool {
        true
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(t("Harness contents"));
    }
}

fn session_types() -> Vec<SessionType> {
    vec![
        SessionType { id: "alpha", name: TRANSLATED, kinds: &[], sides: &[SIDE], new: || Box::new(HarnessView(Session::new("alpha"))) },
        SessionType { id: "beta", name: UNTRANSLATED, kinds: &[], sides: &[], new: || Box::new(HarnessView(Session::new("beta"))) },
    ]
}

/// Options a test feature contributes to alpha's Comparison tab.
fn settings() -> Vec<Setting> {
    let ignore_case = Setting { session_type: "alpha", tab: TAB, id: "ignore_case", label: LABEL, default: Value::Bool(false), choices: &[] };
    let unit = Setting { id: "unit", label: CHOICE_LABEL, default: Value::Choice(CHOICE.into()), choices: &[CHOICE, "Words"], ..ignore_case.clone() };
    vec![ignore_case, unit]
}

fn harness(session_types: Vec<SessionType>) -> Harness<'static, Shell> {
    // egui's default fonts have no `⟦`, the pseudo locale's mark.
    // A command with a toolbar button, so that the toolbar is drawn too.
    let wrap = OptionSpec { page: &[PAGE, SUBPAGE], id: "wrap", label: OPTION, default: Value::Bool(false), choices: &[] };
    let shell = Shell::new(session_types)
        .with_settings(settings())
        .unwrap()
        .with_options(vec![wrap], Default::default())
        .unwrap()
        .with_commands(vec![Command::new("test.tool", "Harness tool", |_| {}).toolbar()]);
    let mut harness = Harness::builder().allow_missing_glyphs().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    harness
}

#[test]
fn a_caption_with_a_translation_shows_translated_and_one_without_shows_its_english_key() {
    let rows = "\tHome\tStart\n\tAlpha Compare\tAlpha-Vergleich\n";
    let catalogs: &'static Embedded = Box::leak(Box::new([("de", "", rows)]));
    let catalogs: &'static I18n = Box::leak(Box::new(I18n::new(catalogs, &["de"]).0));
    set_language(catalogs, catalogs.from_code("de").unwrap_or(Lang::EN));
    let harness = harness(session_types());

    let tab = harness.get_by_role(Role::Tab).accesskit_node().label().unwrap_or_default();
    assert_ne!(tab, HOME, "the shell's own caption is translated");
    assert!(harness.query_by_role_and_label(Role::TreeItem, TRANSLATED).is_none(), "a session type's name is translated");
    assert!(harness.query_by_role_and_label(Role::TreeItem, "Alpha-Vergleich").is_some());
    assert!(harness.query_by_role_and_label(Role::TreeItem, UNTRANSLATED).is_some(), "without a translation, the English key");
}

/// Every label and value in the window, with its role.
pub(crate) fn texts(harness: &Harness<'_, Shell>) -> Vec<(Role, String)> {
    harness
        .query_all_by(|node| node.label().or_else(|| node.value()).is_some_and(|text| !text.is_empty()))
        .map(|node| {
            let node = node.accesskit_node();
            (node.role(), node.label().or_else(|| node.value()).unwrap_or_default())
        })
        .collect()
}

/// `text` is one looked-up string, with any looked-up values inside it: `⟦Close Tab ⟦Home⟧⟧`, not
/// `⟦Result⟧: ⟦Identical⟧`, whose `: ` skipped the lookup.
pub(crate) fn wholly_marked(text: &str) -> bool {
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        match c {
            '⟦' => depth += 1,
            '⟧' if depth > 0 => depth -= 1,
            _ if depth == 0 => return false,
            _ => {}
        }
        if depth == 0 {
            return i + c.len_utf8() == text.len();
        }
    }
    false
}

#[test]
fn no_text_in_the_shell_skips_the_lookup() {
    set_language(embedded(), Lang::PSEUDO);
    // Home without session types shows its notice; Home with them, its buttons; a view, the
    // status bar.
    let mut empty = harness(Vec::new());
    let mut seen = texts(&empty);
    empty.state_mut().open(Box::new(HarnessView(Session::new("none"))));
    empty.run();
    seen.extend(texts(&empty));
    seen.extend(texts(&harness(session_types())));
    // The Session Settings dialog: its Inputs tab, then a contributed tab.
    let mut dialog = harness(session_types());
    dialog.state_mut().new_session("alpha").unwrap();
    dialog.state_mut().session_settings().unwrap();
    dialog.run();
    seen.extend(texts(&dialog));
    dialog.get_by_role_and_label(Role::Tab, &format!("⟦{TAB}⟧")).click();
    dialog.run();
    seen.extend(texts(&dialog));
    // The unsaved-changes prompt, after a Save that failed.
    let mut prompted = harness(Vec::new());
    prompted.state_mut().open(Box::new(UnsavedView));
    prompted.state_mut().close_tab();
    prompted.run();
    prompted.key_press(egui::Key::Enter);
    prompted.run();
    assert!(prompted.query_by_role(Role::AlertDialog).is_some(), "the prompt stays open after the failed Save");
    seen.extend(texts(&prompted));
    // The Options dialog: a page, then the page above it, which has no options of its own.
    let mut options = harness(session_types());
    assert!(options.state_mut().run("otd.options"));
    options.run();
    seen.extend(texts(&options));
    options.get_by_role_and_label(Role::TreeItem, &format!("⟦{PAGE}⟧")).click();
    options.run();
    seen.extend(texts(&options));

    let roles = |role| seen.iter().filter(|(r, _)| *r == role).count();
    assert!(roles(Role::Tab) > 0 && roles(Role::Button) > 0 && roles(Role::Label) > 0, "{seen:?}");
    let unmarked: Vec<_> = seen.iter().filter(|(_, text)| !wholly_marked(text)).collect();
    assert!(unmarked.is_empty(), "text drawn without the lookup: {unmarked:?}");
    // A key filled into a looked-up template, such as a field's label, is marked only inside it.
    let looked_up = embedded().looked_up();
    let keys = [HOME, NEW_TAB, CLOSE_TAB, NO_SESSION_TYPES, INSERT, LOAD_TIME, SESSION_SETTINGS, INPUTS, DESCRIPTION, SCOPE, THIS_VIEW_ONLY];
    let dialog_keys = [OK, CANCEL, SIDE, TAB, LABEL, CHOICE_LABEL, CHOICE, OPTIONS, OPTION_PAGES, APPLY, CHOOSE_A_PAGE, PAGE, SUBPAGE, OPTION];
    let prompt_keys = [SAVE_CHANGES, UNSAVED_LOST, SAVE, DONT_SAVE, CANT_SAVE];
    let skipped: Vec<_> = keys.into_iter().chain(dialog_keys).chain(prompt_keys).filter(|key| !looked_up.contains(*key)).collect();
    assert!(skipped.is_empty(), "drawn without the lookup: {skipped:?}");
}
