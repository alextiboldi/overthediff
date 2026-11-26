//! Opening files into Text Compare (E07-S1-T2): the path edits typed by keyboard, *File > Open File*
//! through a stand-in for the OS picker, and the command line, read back through the view and the
//! accessibility tree.

use std::path::{Path, PathBuf};

use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::session::Value;
use otd_launch::{Config, EXIT_UNCLAIMED, Launch, Registry, Requested};
use otd_settings::{Location, Store};
use otd_ui::path_edit::{Answer, Request};
use otd_ui::{PathEdits, Shell};
use otd_ui_text_compare::{LEFT, OpenFiles, RIGHT, SESSION_TYPE, TEXT_COMPARE, TextCompareView, commands};
use raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

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

/// The main window as the app drives it: the shell, then the path edits' pickers.
struct Window {
    shell: Shell,
    edits: Option<PathEdits>,
    store: Store,
}

fn shell() -> Shell {
    Shell::new(vec![SESSION_TYPE]).with_commands(commands())
}

fn harness(shell: Shell) -> Harness<'static, Window> {
    let (store, _) = otd_settings::open(Location::Memory);
    let mut harness = Harness::builder().build_ui_state(
        |ui, window: &mut Window| {
            window.shell.ui(ui);
            if let Some(edits) = &mut window.edits {
                edits.frame::<NoWindow>(ui.ctx(), None, &mut window.store);
            }
        },
        Window { shell, edits: None, store },
    );
    let ctx = harness.ctx.clone();
    let window = harness.state_mut();
    window.edits = Some(PathEdits::open(&ctx, &mut window.store));
    harness.run();
    harness
}

fn view<'a>(harness: &'a Harness<'_, Window>) -> &'a TextCompareView {
    harness.state().shell.view::<TextCompareView>().expect("a Text Compare tab is active")
}

fn press(harness: &mut Harness<'_, Window>, key: Key) {
    harness.key_press_modifiers(Modifiers::NONE, key);
    harness.run();
}

fn focused(harness: &Harness<'_, Window>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// Moves focus with Tab to the path edit named `side`.
fn tab_to_edit(harness: &mut Harness<'_, Window>, side: &str) {
    for _ in 0..40 {
        if focused(harness) == Some((Role::TextInput, side.to_owned())) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached the {side} path edit");
}

fn write(folder: &Path, name: &str, text: &str) -> PathBuf {
    let path = folder.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn a_typed_path_loads_into_its_pane_on_enter_and_a_missing_file_shows_its_error_in_that_pane() {
    let folder = tempfile::tempdir().unwrap();
    let missing = folder.path().join("missing.txt").display().to_string();
    let right = write(folder.path(), "right.txt", "right text\n").display().to_string();
    let mut harness = harness(shell());
    harness.state_mut().shell.new_session(TEXT_COMPARE).unwrap();
    harness.run();

    tab_to_edit(&mut harness, LEFT);
    harness.get_by(|node| node.is_focused()).type_text(&missing);
    harness.run();
    assert_eq!(view(&harness).error(0), None, "a half-typed path isn't loaded");
    press(&mut harness, Key::Enter);
    let error = view(&harness).error(0).expect("the missing file's error").to_owned();
    assert!(view(&harness).pane(0).is_none(), "the pane has no document, so nothing to edit");
    let shown = harness.get_by_label(&error);
    assert_eq!(shown.accesskit_node().live(), Live::Polite, "the error is announced");

    tab_to_edit(&mut harness, RIGHT);
    harness.get_by(|node| node.is_focused()).type_text(&right);
    harness.run();
    press(&mut harness, Key::Enter);
    assert_eq!(view(&harness).pane(1).map(|pane| pane.path.as_str()), Some(right.as_str()));
    assert!(harness.query_by_label("right text").is_some(), "the right pane shows its file");
    assert!(view(&harness).error(0).is_some(), "the left pane keeps its own error");

    // Once the file exists, using the same path again retries it.
    std::fs::write(&missing, "now here\n").unwrap();
    let answer = PathBuf::from(&missing);
    harness.state_mut().edits.as_mut().unwrap().set_override(Some(Box::new(move |_: &Request| Answer::Paths(vec![answer.clone()]))));
    tab_to_edit(&mut harness, LEFT);
    assert!(harness.state_mut().shell.run("text-compare.open_file"));
    for _ in 0..3 {
        harness.run();
    }
    assert_eq!(view(&harness).pane(0).map(|pane| pane.path.as_str()), Some(missing.as_str()), "the same path, browsed again, loads");
}

#[test]
fn open_file_loads_into_the_chosen_pane_through_its_picker_or_from_a_path_argument() {
    let folder = tempfile::tempdir().unwrap();
    let picked = write(folder.path(), "picked.txt", "picked\n");
    let given = write(folder.path(), "given.txt", "given\n").display().to_string();
    let mut harness = harness(shell());
    harness.state_mut().shell.new_session(TEXT_COMPARE).unwrap();
    harness.run();
    let answer = picked.clone();
    harness.state_mut().edits.as_mut().unwrap().set_override(Some(Box::new(move |_: &Request| Answer::Paths(vec![answer.clone()]))));

    // The right pane is chosen by focusing its path edit.
    tab_to_edit(&mut harness, RIGHT);
    assert_eq!(view(&harness).chosen(), 1);
    assert!(harness.state_mut().shell.run("text-compare.open_file"));
    for _ in 0..3 {
        harness.run();
    }
    assert_eq!(view(&harness).pane(1).map(|pane| pane.path.clone()), Some(picked.display().to_string()), "the picked file is in the chosen pane");
    assert!(view(&harness).pane(0).is_none(), "the other pane is untouched");

    // Without a picker: the hidden command, given the side and the path.
    let commands = commands();
    let command = commands.iter().find(|c| c.id == "text-compare.open_file_on_file_system").unwrap();
    let shell = &mut harness.state_mut().shell;
    let args = [("side", Value::Choice(LEFT.to_owned())), ("path", Value::Path(given.clone()))];
    command.bind(shell, &args, true).unwrap().run(shell).unwrap();
    harness.run();
    assert_eq!(view(&harness).pane(0).map(|pane| pane.path.as_str()), Some(given.as_str()));
}

fn registry() -> Registry<Shell> {
    let mut registry = Registry::new();
    registry.register(OpenFiles);
    registry
}

fn config() -> Config {
    Config { app_version: "1.0.0".into(), runtime: None, spawn_instance: Box::new(|| Ok(())) }
}

#[test]
fn two_files_on_the_command_line_open_one_text_compare_tab_relative_to_the_launch_folder() {
    let folder = tempfile::tempdir().unwrap();
    // A name starting with `~` is a file in the launch folder, not a home folder.
    let left = write(folder.path(), "~left.txt", "left\n");
    let right = write(folder.path(), "right.txt", "right\n");
    let registry = registry();
    let args = ["~left.txt".to_owned(), "right.txt".to_owned()];
    // A new instance of its own, so the test reaches no running app.
    let Launch::Instance(instance) = otd_launch::launch(&registry, &args, Requested::NewInstance, &config()) else { panic!("the launch is refused") };
    let mut incoming = instance.try_next().unwrap();
    // The launching process's folder, which differs from the instance's.
    incoming.request.cwd = folder.path().to_path_buf();

    let mut shell = shell();
    registry.open(incoming, &mut shell).unwrap();
    let harness = harness(shell);
    assert_eq!(harness.query_all_by_role(Role::Tab).count(), 2, "Home and one Text Compare tab");
    let paths: Vec<_> = (0..2).map(|side| view(&harness).pane(side).map(|pane| pane.path.clone())).collect();
    assert_eq!(paths, [Some(left.display().to_string()), Some(right.display().to_string())]);
}

#[test]
fn a_command_line_with_other_than_two_files_is_refused_with_the_usage() {
    for args in [&["a.txt"][..], &["a.txt", "b.txt", "c.txt"]] {
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
        match otd_launch::launch(&registry(), &args, Requested::NewInstance, &config()) {
            Launch::Exit { code, message } => {
                assert_eq!(code, EXIT_UNCLAIMED, "{args:?}");
                assert!(message.is_some_and(|m| m.contains("Usage")), "{args:?} prints the usage");
            }
            Launch::Instance(_) => panic!("{args:?} is accepted"),
        }
    }
}
