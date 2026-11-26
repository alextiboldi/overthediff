//! Tree scanning: opened folders first, the rest in the background, scan states and progress.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime};

use otd_folder_engine::{ScanEvent, ScanState, TreeScan};
use otd_source::{Capabilities, DataSource, ErrorKind, MemorySource};

/// Only a guard against a hung worker; no test asserts on time.
const DEADLINE: Duration = Duration::from_secs(30);

fn path(names: &[&str]) -> PathBuf {
    names.iter().collect()
}

/// `a`, `a/sub`, `b`, `c`, `c/deep` and `c/deep/deeper`, plus files, a link to `c` and a container.
fn tree() -> Arc<MemorySource> {
    let caps = Capabilities {
        read_only: false,
        case_sensitive: true,
        recycle_bin: false,
        timestamp_resolution: Duration::ZERO,
        windows_attributes: false,
        unix_permissions: false,
    };
    let t = SystemTime::UNIX_EPOCH;
    Arc::new(
        MemorySource::new(caps)
            .file("top.txt", "x", t)
            .file("a/sub/one.txt", "1", t)
            .folder("b", t)
            .file("c/deep/deeper/two.txt", "2", t)
            .link("to-c", "c", t)
            .container("pack.zip", "zip", t)
            .file("pack.zip/inside.txt", "i", t),
    )
}

/// The folders `tree` holds, the base included.
fn folders() -> Vec<PathBuf> {
    vec![path(&[]), path(&["a"]), path(&["a", "sub"]), path(&["b"]), path(&["c"]), path(&["c", "deep"]), path(&["c", "deep", "deeper"])]
}

fn state(scan: &TreeScan, folder: &Path) -> Option<ScanState> {
    scan.folder(folder).map(|f| f.state)
}

fn listed(events: &Receiver<ScanEvent>) -> Vec<PathBuf> {
    events.try_iter().map(|event| event.folder).collect()
}

#[test]
fn an_opened_folder_is_listed_before_folders_queued_ahead_of_it() {
    let (scan, events) = TreeScan::new(tree(), "", true);
    assert!(scan.step(), "the base is listed first");
    assert_eq!(state(&scan, &path(&["a"])), Some(ScanState::Pending), "background scanning queued a");
    scan.open(&path(&["c"]));
    assert!(scan.step());
    assert_eq!(listed(&events), [path(&[]), path(&["c"])]);
}

#[test]
fn with_background_scanning_off_a_folder_stays_unscanned_until_opened_then_goes_pending_then_done() {
    let (scan, events) = TreeScan::new(tree(), "", false);
    while scan.step() {}
    let a = path(&["a"]);
    assert_eq!(state(&scan, &a), Some(ScanState::Unscanned));
    scan.open(&a);
    assert_eq!(state(&scan, &a), Some(ScanState::Pending));
    assert!(scan.step());
    assert_eq!(state(&scan, &a), Some(ScanState::Done));
    assert_eq!(state(&scan, &path(&["a", "sub"])), Some(ScanState::Unscanned), "opening a folder doesn't scan below it");
    assert!(!scan.step(), "nothing else was asked for");
    assert_eq!(listed(&events), [path(&[]), a]);
}

#[test]
fn background_scanning_lists_every_folder_once_and_its_progress_ends_with_nothing_pending() {
    let (scan, events) = TreeScan::new(tree(), "", true);
    scan.open(&path(&["c", "deep"]));
    while scan.step() {}
    let events: Vec<ScanEvent> = events.try_iter().collect();
    let mut listed: Vec<PathBuf> = events.iter().map(|e| e.folder.clone()).collect();
    listed.sort();
    let mut expected = folders();
    expected.sort();
    assert_eq!(listed, expected, "every folder, and no link or unopened container, is listed exactly once");
    assert!(events.windows(2).all(|w| w[1].done > w[0].done), "done only grows: {events:?}");
    let last = events.last().expect("the scan listed something");
    assert_eq!((last.done, last.pending), (folders().len(), 0));
    assert!(events[..events.len() - 1].iter().all(|e| e.pending > 0), "nothing pending is reported only at the end: {events:?}");
}

#[test]
fn a_folder_that_cannot_be_listed_ends_done_with_its_error_and_the_rest_of_the_tree_still_scans() {
    let source = MemorySource::clone(&tree()).unreadable("a");
    let (scan, _events) = TreeScan::new(Arc::new(source), "", true);
    while scan.step() {}
    let a = scan.folder(&path(&["a"])).expect("a was found");
    let error = a.error.expect("a keeps why it couldn't be listed");
    assert!(a.state == ScanState::Done && a.items.is_empty() && error.kind == ErrorKind::AccessDenied && !error.message.is_empty());
    assert_eq!(state(&scan, &path(&["c", "deep", "deeper"])), Some(ScanState::Done));
}

#[test]
fn a_scan_of_a_base_folder_reports_paths_relative_to_it() {
    let (scan, events) = TreeScan::new(tree(), "c", true);
    while scan.step() {}
    let mut listed = listed(&events);
    listed.sort();
    assert_eq!(listed, [path(&[]), path(&["deep"]), path(&["deep", "deeper"])]);
}

#[test]
fn the_worker_completes_the_tree_in_the_background() {
    let (scan, events) = TreeScan::new(tree(), "", true);
    scan.start().expect("the worker starts");
    while events.recv_timeout(DEADLINE).expect("the worker keeps listing").pending > 0 {}
    assert!(folders().iter().all(|f| state(&scan, f) == Some(ScanState::Done)));
}

#[test]
fn an_idle_worker_lists_a_folder_as_soon_as_it_is_opened() {
    let (scan, events) = TreeScan::new(tree(), "", false);
    scan.start().expect("the worker starts");
    assert_eq!(events.recv_timeout(DEADLINE).expect("the base is listed").folder, path(&[]));
    scan.open(&path(&["b"]));
    assert_eq!(events.recv_timeout(DEADLINE).expect("the opened folder is listed").folder, path(&["b"]));
}

#[test]
fn refresh_lists_a_listed_folder_again_with_what_changed_and_below_it_only_folders_already_listed() {
    let source = tree();
    let (scan, events) = TreeScan::new(Arc::clone(&source) as Arc<dyn DataSource>, "", false);
    scan.open(&path(&["a"]));
    while scan.step() {}
    let _ = listed(&events);
    let names = |folder: &[&str]| scan.folder(&path(folder)).map(|f| f.items.into_iter().map(|i| i.name).collect::<Vec<_>>());
    let before = (names(&[]), names(&["a"]));
    source.write(&path(&["new.txt"]), &mut "n".as_bytes()).unwrap();
    source.write(&path(&["a", "new.txt"]), &mut "n".as_bytes()).unwrap();
    assert_eq!((names(&[]), names(&["a"])), before, "nothing changes until refreshed");

    assert_eq!(scan.refresh(&path(&[]), false), 1, "only the base itself");
    while scan.step() {}
    assert!(names(&[]).unwrap().contains(&"new.txt".to_owned()), "the base shows its new file");
    assert_eq!(names(&["a"]), before.1, "a wasn't listed again");

    scan.refresh(&path(&[]), true);
    while scan.step() {}
    let mut again = listed(&events);
    again.sort();
    assert_eq!(again, [path(&[]), path(&[]), path(&["a"])], "every listed folder, and no folder never opened, such as c");
    assert!(names(&["a"]).unwrap().contains(&"new.txt".to_owned()), "a shows its new file");
    assert_eq!(state(&scan, &path(&["c"])), Some(ScanState::Unscanned));
}
