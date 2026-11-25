//! The shell's text goes through the translation lookup (E01-S1-T13): a caption with a test
//! translation shows translated, one without shows its English key, and in the pseudo locale,
//! which marks every looked-up string `⟦…⟧`, every text in the window is one marked string.

use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::{Embedded, I18n, Lang, embedded, set_language, t};
use otd_ui::{CLOSE_TAB, Field, HOME, INSERT, LOAD_TIME, NEW_TAB, NO_SESSION_TYPES, SessionType, Shell, Status, View};

const TRANSLATED: &str = "Alpha Compare";
const UNTRANSLATED: &str = "Beta Compare";

/// A test view whose own text, like a real view's, is looked up.
struct HarnessView;

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

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(t("Harness contents"));
    }
}

fn session_types() -> Vec<SessionType> {
    vec![
        SessionType { id: "alpha", name: TRANSLATED, new: || Box::new(HarnessView) },
        SessionType { id: "beta", name: UNTRANSLATED, new: || Box::new(HarnessView) },
    ]
}

fn harness(session_types: Vec<SessionType>) -> Harness<'static, Shell> {
    // egui's default fonts have no `⟦`, the pseudo locale's mark.
    let mut harness = Harness::builder().allow_missing_glyphs().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(session_types));
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
    assert!(harness.query_by_role_and_label(Role::Button, TRANSLATED).is_none(), "a session type's name is translated");
    assert!(harness.query_by_role_and_label(Role::Button, "Alpha-Vergleich").is_some());
    assert!(harness.query_by_role_and_label(Role::Button, UNTRANSLATED).is_some(), "without a translation, the English key");
}

/// Every label and value in the window, with its role.
fn texts(harness: &Harness<'_, Shell>) -> Vec<(Role, String)> {
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
fn wholly_marked(text: &str) -> bool {
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
    empty.state_mut().open(Box::new(HarnessView));
    empty.run();
    seen.extend(texts(&empty));
    seen.extend(texts(&harness(session_types())));

    let roles = |role| seen.iter().filter(|(r, _)| *r == role).count();
    assert!(roles(Role::Tab) > 0 && roles(Role::Button) > 0 && roles(Role::Label) > 0, "{seen:?}");
    let unmarked: Vec<_> = seen.iter().filter(|(_, text)| !wholly_marked(text)).collect();
    assert!(unmarked.is_empty(), "text drawn without the lookup: {unmarked:?}");
    // A key filled into a looked-up template, such as a field's label, is marked only inside it.
    let looked_up = embedded().looked_up();
    let skipped: Vec<_> =
        [HOME, NEW_TAB, CLOSE_TAB, NO_SESSION_TYPES, INSERT, LOAD_TIME].into_iter().filter(|key| !looked_up.contains(*key)).collect();
    assert!(skipped.is_empty(), "drawn without the lookup: {skipped:?}");
}
