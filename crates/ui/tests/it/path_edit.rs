//! The path edit, driven by keyboard and read back through the accessibility tree: recent paths
//! kept per field through a real settings store and a restart, and Browse through a stand-in for
//! the OS picker (`PathEdits::set_override`), so no native dialog opens.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use craft_testkit::hostile::{HammerConfig, hammer};
use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_settings::{Location, Store};
use otd_ui::path_edit::{Answer, Override, PickerKind, Request};
use otd_ui::{Browse, PathEdit, PathEdits};
use raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

const LEFT: &str = "Left";
const RIGHT: &str = "Right";

/// What `PathEdits::frame` gets as the parent in a test: no window, which the override never needs.
struct NoWindow;

impl HasWindowHandle for NoWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for NoWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

/// A window with two path edits, as the app drives them.
struct Window {
    edits: Option<PathEdits>,
    store: Store,
    left: String,
    right: String,
    /// The Left edit isn't drawn, as when the dialog or tab holding it has closed.
    hide_left: Arc<AtomicBool>,
}

fn harness(store: Store) -> Harness<'static, Window> {
    let window = Window { edits: None, store, left: String::new(), right: String::new(), hide_left: Arc::default() };
    let mut harness = Harness::builder().build_ui_state(
        |ui, window: &mut Window| {
            for (label, field, path) in [(LEFT, "test.left", &mut window.left), (RIGHT, "test.right", &mut window.right)] {
                if window.hide_left.load(Ordering::SeqCst) && label == LEFT {
                    continue;
                }
                ui.horizontal(|ui| {
                    let label = ui.label(label);
                    PathEdit::new(field, path, Browse::Both).show(ui).labelled_by(label.id);
                });
            }
            if let Some(edits) = &mut window.edits {
                edits.frame::<NoWindow>(ui.ctx(), None, &mut window.store);
            }
        },
        window,
    );
    let ctx = harness.ctx.clone();
    let window = harness.state_mut();
    window.edits = Some(PathEdits::open(&ctx, &mut window.store));
    harness.run();
    harness
}

fn answer_with(harness: &mut Harness<'_, Window>, answer: Override) {
    harness.state_mut().edits.as_mut().unwrap().set_override(Some(answer));
}

fn press(harness: &mut Harness<'_, Window>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn focused(harness: &Harness<'_, Window>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().or_else(|| node.value()).unwrap_or_default()))
}

fn tab_to(harness: &mut Harness<'_, Window>, role: Role, name: &str) {
    for _ in 0..30 {
        if focused(harness).is_some_and(|(r, label)| r == role && label == name) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached {name:?}");
}

/// Types `text` into the edit labelled `label` and leaves it with Tab.
fn type_path(harness: &mut Harness<'_, Window>, label: &str, text: &str) {
    tab_to(harness, Role::TextInput, label);
    harness.get_by(|node| node.is_focused()).type_text(text);
    harness.run();
    press(harness, Key::Tab);
}

/// Opens the first Recent paths dropdown (the Left edit's) and returns the paths it lists.
fn open_left_recent(harness: &mut Harness<'_, Window>) -> Vec<String> {
    tab_to(harness, Role::ComboBox, otd_ui::path_edit::RECENT_PATHS);
    press(harness, Key::Space);
    harness
        .query_all_by(|node| node.parent().is_some_and(|parent| parent.role() == Role::ListBox) || node.role() == Role::Button)
        .filter_map(|node| node.accesskit_node().label())
        .filter(|label| label.starts_with("/"))
        .collect()
}

#[test]
fn recent_paths_persist_per_field_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let (store, _) = otd_settings::open(Location::Dir(dir.path().to_path_buf()));
    let mut before = harness(store);
    // Right first, so a lookup that ignored the field would find Right's list first.
    type_path(&mut before, RIGHT, "/right/b.txt");
    type_path(&mut before, LEFT, "/left/a.txt");
    before.state_mut().store.flush().unwrap();
    drop(before);

    let (store, notice) = otd_settings::open(Location::Dir(dir.path().to_path_buf()));
    assert_eq!(notice, None);
    let mut after = harness(store);
    assert_eq!(open_left_recent(&mut after), ["/left/a.txt"], "the Left field's path, and not the Right one's");
}

#[test]
fn a_recent_path_picked_from_the_dropdown_fills_the_edit_and_focus_returns_to_it() {
    let (store, _) = otd_settings::open(Location::Memory);
    let mut harness = harness(store);
    type_path(&mut harness, LEFT, "/left/a.txt");
    harness.state_mut().left.clear();
    harness.run();
    assert!(open_left_recent(&mut harness).contains(&"/left/a.txt".to_owned()));
    tab_to(&mut harness, Role::Button, "/left/a.txt");
    press(&mut harness, Key::Enter);
    assert_eq!(harness.state().left, "/left/a.txt");
    assert_eq!(focused(&harness).map(|(role, _)| role), Some(Role::TextInput));
}

#[test]
fn browse_asks_the_picker_for_a_file_or_a_folder_and_returns_the_chosen_path_into_the_edit() {
    let (store, _) = otd_settings::open(Location::Memory);
    let mut harness = harness(store);
    let asked = Arc::new(Mutex::new(Vec::new()));
    let log = asked.clone();
    answer_with(
        &mut harness,
        Box::new(move |request: &Request| {
            let Request::File(dialog) = request else { return Answer::Cancelled };
            log.lock().unwrap().push(dialog.kind);
            Answer::Paths(vec![PathBuf::from(if dialog.kind == PickerKind::Folder { "/chosen/folder" } else { "/chosen/file.txt" })])
        }),
    );

    tab_to(&mut harness, Role::Button, otd_ui::path_edit::BROWSE_FOR_FILE);
    press(&mut harness, Key::Enter);
    assert_eq!(harness.state().left, "/chosen/file.txt");
    assert_eq!(focused(&harness), Some((Role::TextInput, LEFT.to_owned())), "focus goes back to the edit");

    tab_to(&mut harness, Role::Button, otd_ui::path_edit::BROWSE_FOR_FOLDER);
    press(&mut harness, Key::Enter);
    assert_eq!(harness.state().left, "/chosen/folder");
    assert_eq!(*asked.lock().unwrap(), [PickerKind::Open { multiple: false }, PickerKind::Folder]);
    assert_eq!(harness.state().right, "", "the other edit is untouched");
}

#[test]
fn a_picker_that_cant_be_shown_is_announced_and_leaves_the_edit_alone() {
    let (store, _) = otd_settings::open(Location::Memory);
    let mut harness = harness(store);
    answer_with(&mut harness, Box::new(|_: &Request| Answer::Unavailable("no desktop portal".into())));
    tab_to(&mut harness, Role::Button, otd_ui::path_edit::BROWSE_FOR_FILE);
    press(&mut harness, Key::Enter);
    let announced: Vec<String> = harness
        .query_all_by(|node| node.live() == Live::Polite)
        .filter_map(|node| node.accesskit_node().label().or(node.accesskit_node().value()))
        .collect();
    assert!(announced.iter().any(|text| text.contains("no desktop portal")), "{announced:?}");
    assert_eq!(harness.state().left, "");
}

#[test]
fn esc_closes_the_recent_paths_list_and_leaves_focus_on_its_dropdown() {
    let (store, _) = otd_settings::open(Location::Memory);
    let mut harness = harness(store);
    type_path(&mut harness, LEFT, "/left/a.txt");
    open_left_recent(&mut harness);
    assert!(harness.query_by_role(Role::ListBox).is_some(), "the list is open");
    press(&mut harness, Key::Escape);
    assert!(harness.query_by_role(Role::ListBox).is_none(), "the list closed");
    assert_eq!(focused(&harness), Some((Role::ComboBox, otd_ui::path_edit::RECENT_PATHS.to_owned())));
}

#[test]
fn a_picker_answer_for_an_edit_no_longer_drawn_never_fills_it_later() {
    let (store, _) = otd_settings::open(Location::Memory);
    let mut harness = harness(store);
    let hide_left = harness.state().hide_left.clone();
    let hide = hide_left.clone();
    // The picker answers as its edit goes away, so from the next frame on nothing draws the edit.
    answer_with(
        &mut harness,
        Box::new(move |_: &Request| {
            hide.store(true, Ordering::SeqCst);
            Answer::Paths(vec![PathBuf::from("/chosen/file.txt")])
        }),
    );
    tab_to(&mut harness, Role::Button, otd_ui::path_edit::BROWSE_FOR_FILE);
    press(&mut harness, Key::Enter);
    assert!(hide_left.swap(false, Ordering::SeqCst), "the picker answered");
    harness.run();
    assert_eq!(harness.state().left, "", "the answer went with the edit it was for");
}

#[test]
fn opening_the_recent_paths_from_a_damaged_settings_file_never_panics_or_hangs() {
    let seed = "schema_version = 1\n\n[[recent-paths.fields]]\nfield = \"test.left\"\npaths = [\"/a.txt\", \"dir/b.txt\"]\n\n[[recent-paths.fields]]\nfield = \"test.right\"\npaths = []\n";
    hammer(&HammerConfig::new("recent paths", 300), seed.as_bytes(), |data| {
        let Ok(dir) = tempfile::tempdir() else { return };
        if std::fs::write(dir.path().join("config.toml"), data).is_err() {
            return;
        }
        let (mut store, _notice) = otd_settings::open(Location::Dir(dir.path().to_path_buf()));
        let _ = PathEdits::open(&egui::Context::default(), &mut store);
    });
}
