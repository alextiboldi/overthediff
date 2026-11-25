//! Background tasks in the shell, driven by keyboard and read back through the accessibility tree:
//! the progress window's Cancel and the end-of-task summary. The task threads are stepped through
//! channels, never by time.

use std::sync::mpsc;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use otd_core::i18n::{Lang, embedded, set_language, t};
use otd_core::task::{End, Task};
use otd_ui::{CANCEL, CANCELLED, CLOSE, Shell, show_task};

fn harness() -> Harness<'static, Shell> {
    // egui's default fonts have no `⟦`, the pseudo locale's mark.
    let mut harness = Harness::builder()
        .with_os(egui::os::OperatingSystem::Mac)
        .allow_missing_glyphs()
        .build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(Vec::new()));
    harness.run();
    harness
}

fn press(harness: &mut Harness<'_, Shell>, key: Key) {
    harness.key_press(key);
    harness.run();
}

fn focused(harness: &Harness<'_, Shell>) -> Option<(Role, String)> {
    let node = harness.query_by(|node| node.is_focused())?;
    let node = node.accesskit_node();
    Some((node.role(), node.label().unwrap_or_default()))
}

/// Presses Tab until the button named `name` has focus.
fn tab_to(harness: &mut Harness<'_, Shell>, name: &str) {
    for _ in 0..20 {
        if focused(harness) == Some((Role::Button, name.to_owned())) {
            return;
        }
        press(harness, Key::Tab);
    }
    panic!("Tab never reached {name}; focus is on {:?}", focused(harness));
}

fn dialog_open(harness: &Harness<'_, Shell>, title: &str) -> bool {
    harness.query_all_by_role(Role::Dialog).any(|node| node.accesskit_node().label().as_deref() == Some(title))
}

/// A task that fails the items `a`, `c` and `e` of five.
fn three_failures() -> Task {
    let task = Task::spawn("Copy", true, |worker| {
        for item in ["a", "b", "c", "d", "e"] {
            if matches!(item, "a" | "c" | "e") {
                worker.fail(item, "Access denied");
            }
        }
    });
    task.wait();
    task
}

#[test]
fn cancel_from_the_keyboard_stops_a_task_midway_and_its_summary_takes_focus() {
    let mut harness = harness();
    let (reached, progress) = mpsc::channel();
    let (go, wait_for_go) = mpsc::channel::<()>();
    let task = Task::spawn("Count", true, move |worker| {
        for done in 1..=10 {
            if worker.cancelled() {
                return;
            }
            worker.progress(done, Some(10));
            let _ = reached.send(done);
            let _ = wait_for_go.recv();
        }
    });
    assert_eq!(progress.recv(), Ok(1));
    show_task(&harness.ctx, task.clone());
    harness.run();
    let bar = harness.query_by_role(Role::ProgressIndicator).expect("a progress bar shows").accesskit_node();
    assert!(bar.label().is_some_and(|label| label.starts_with("1 of 10")), "its name gives the progress: {:?}", bar.label());

    tab_to(&mut harness, CANCEL);
    press(&mut harness, Key::Enter);
    assert!(task.cancel_requested());
    go.send(()).unwrap();
    let outcome = task.wait();
    assert_eq!(outcome.end, End::Cancelled);
    assert!(outcome.progress.done < 10, "it stopped midway: {:?}", outcome.progress);
    harness.run();

    assert!(harness.query_by_role(Role::ProgressIndicator).is_none(), "the progress window closed");
    assert!(dialog_open(&harness, "Count"), "the summary shows");
    assert!(harness.query_by_label(CANCELLED).is_some(), "the summary says the task was cancelled");
    assert_eq!(focused(&harness), Some((Role::Button, CLOSE.to_owned())));
    press(&mut harness, Key::Escape);
    assert!(!dialog_open(&harness, "Count"), "Esc closes the summary");
}

#[test]
fn a_task_with_three_failing_items_lists_all_three_in_its_summary() {
    let mut harness = harness();
    show_task(&harness.ctx, three_failures());
    harness.run();
    for item in ["a", "c", "e"] {
        let line = format!("{item}: Access denied");
        assert!(harness.query_by_label(&line).is_some(), "the summary lists {line:?}");
    }
    assert!(harness.query_by_label("b: Access denied").is_none());
}

#[test]
fn closing_the_summary_puts_focus_back_where_it_was_when_the_task_started() {
    let mut harness = harness();
    press(&mut harness, Key::Tab);
    let before = focused(&harness);
    assert!(before.is_some(), "something has focus before the task starts");
    show_task(&harness.ctx, three_failures());
    harness.run();
    assert_eq!(focused(&harness), Some((Role::Button, CLOSE.to_owned())));
    press(&mut harness, Key::Enter);
    assert!(!dialog_open(&harness, "Copy"));
    assert_eq!(focused(&harness), before);
}

#[test]
fn a_clean_completion_gives_focus_back_from_its_cancel_button() {
    let mut harness = harness();
    press(&mut harness, Key::Tab);
    let before = focused(&harness);
    assert!(before.is_some(), "something has focus before the task starts");
    let (finish, wait_for_finish) = mpsc::channel::<()>();
    let task = Task::spawn("Count", true, move |_| {
        let _ = wait_for_finish.recv();
    });
    show_task(&harness.ctx, task.clone());
    harness.run();
    tab_to(&mut harness, CANCEL);
    finish.send(()).unwrap();
    assert_eq!(task.wait().end, End::Completed);
    harness.run();
    assert!(harness.query_by_role(Role::ProgressIndicator).is_none(), "the progress window closed");
    assert_eq!(focused(&harness), before);
}

#[test]
fn tab_shortcuts_do_nothing_behind_the_summary() {
    let mut harness = harness();
    show_task(&harness.ctx, three_failures());
    harness.run();
    let tabs = harness.query_all_by_role(Role::Tab).count();
    harness.key_press_modifiers(egui::Modifiers::COMMAND, Key::T);
    harness.run();
    assert_eq!(harness.query_all_by_role(Role::Tab).count(), tabs, "Cmd+T opened no tab");
    assert!(dialog_open(&harness, "Copy"), "the summary is still open");
}

#[test]
fn showing_one_task_twice_shows_it_once() {
    let mut harness = harness();
    let task = three_failures();
    show_task(&harness.ctx, task.clone());
    show_task(&harness.ctx, task);
    harness.run();
    press(&mut harness, Key::Escape);
    assert!(!dialog_open(&harness, "Copy"), "one Esc closed its only summary");
}

#[test]
fn no_text_in_the_task_windows_skips_the_lookup() {
    set_language(embedded(), Lang::PSEUDO);
    let mut harness = harness();
    // The feature looks its title and messages up on the UI thread; the task's thread reads English.
    let (title, message) = (t("Copy").to_owned(), t("Access denied").to_owned());
    let failed = Task::spawn(title, true, move |worker| {
        worker.fail("a", message.as_str());
        worker.fail("b", message.as_str());
    });
    failed.wait();
    let (reached, progress) = mpsc::channel();
    let (finish, wait_for_finish) = mpsc::channel::<()>();
    let running = Task::spawn(t("Count"), true, move |worker| {
        worker.progress(1, Some(2));
        let _ = reached.send(());
        let _ = wait_for_finish.recv();
    });
    progress.recv().unwrap();
    show_task(&harness.ctx, running.clone());
    harness.run();
    let mut seen = crate::translation::texts(&harness);
    show_task(&harness.ctx, failed);
    harness.run();
    seen.extend(crate::translation::texts(&harness));
    finish.send(()).unwrap();
    running.wait();

    let roles = |role| seen.iter().filter(|(r, _)| *r == role).count();
    assert!(roles(Role::ProgressIndicator) > 0 && roles(Role::Dialog) > 0, "{seen:?}");
    let unmarked: Vec<_> = seen.iter().filter(|(_, text)| !crate::translation::wholly_marked(text)).collect();
    assert!(unmarked.is_empty(), "text drawn without the lookup: {unmarked:?}");
}
