//! Headless quick compare (E11-S2) through the real `otd` executable: it exits by itself with the
//! comparison's code, so it never waits on a window.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use craft_testkit::with_deadline;

const PREFIX: &str = if cfg!(windows) { "/" } else { "-" };

/// Runs `otd <prefix>qc=<test> <left> <right>` in `dir` and returns its exit code and standard
/// error. A run that opened a window would block, so one still running after a minute fails.
fn qc(test: &str, left: &str, right: &str, dir: &Path) -> (Option<i32>, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_otd"));
    command.args([&format!("{PREFIX}qc={test}"), left, right]).current_dir(dir).stdin(Stdio::null());
    let output = with_deadline(Duration::from_secs(60), "otd -qc exits by itself", move || command.output().expect("run otd"));
    (output.status.code(), String::from_utf8_lossy(&output.stderr).into_owned())
}

#[test]
fn quick_compare_exits_with_the_comparison_code_and_no_window() {
    let dir = tempfile::tempdir().expect("temporary folder");
    std::fs::write(dir.path().join("a"), b"same size").expect("write a");
    std::fs::write(dir.path().join("b"), b"same size").expect("write b");
    std::fs::write(dir.path().join("c"), b"some size").expect("write c");
    std::fs::write(dir.path().join("d"), b"a longer size").expect("write d");
    for test in ["size", "crc", "binary"] {
        assert_eq!(qc(test, "a", "b", dir.path()).0, Some(1), "identical files by {test}");
        assert_eq!(qc(test, "a", "d", dir.path()).0, Some(11), "different sizes by {test}");
    }
    assert_eq!(qc("size", "a", "c", dir.path()).0, Some(1), "equal sizes by size");
    assert_eq!(qc("crc", "a", "c", dir.path()).0, Some(11), "different bytes by crc");
    assert_eq!(qc("binary", "a", "c", dir.path()).0, Some(11), "different bytes by binary");
    let (code, stderr) = qc("binary", "a", "missing", dir.path());
    assert_eq!(code, Some(100), "a missing file");
    assert_eq!(stderr.trim_end().lines().count(), 1, "one line of error: {stderr:?}");
}
