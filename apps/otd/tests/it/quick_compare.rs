//! Headless quick compare (E11-S2) through the real `otd` executable: it exits by itself with the
//! comparison's code, so it never waits on a window.

use std::io::Read as _;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const PREFIX: &str = if cfg!(windows) { "/" } else { "-" };

/// Runs `otd <prefix>qc=<test> <left> <right>` in `dir` and returns its exit code and standard
/// error. A run that opened a window would block, so one still running after a minute fails.
fn qc(test: &str, left: &str, right: &str, dir: &std::path::Path) -> (Option<i32>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_otd"))
        .args([&format!("{PREFIX}qc={test}"), left, right])
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start otd");
    // ponytail: polls try_wait; the deadline only turns a hang into a failure.
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait for otd") {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("otd {PREFIX}qc={test} didn't exit by itself");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let mut stderr = String::new();
    child.stderr.take().expect("piped").read_to_string(&mut stderr).expect("read standard error");
    (status.code(), stderr)
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
