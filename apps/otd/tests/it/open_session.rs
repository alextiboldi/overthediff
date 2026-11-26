//! `otd <name>` through the real `otd` executable (E06-S1-T7). Only the refusal runs here: a known
//! name would hand off to a running instance or open a window.

use std::process::{Command, Stdio};
use std::time::Duration;

use craft_testkit::with_deadline;

#[test]
fn an_unknown_session_name_gives_the_usage_error_and_exit_code_100() {
    let settings = tempfile::tempdir().expect("temporary settings folder");
    let cwd = tempfile::tempdir().expect("temporary working folder");
    let mut command = Command::new(env!("CARGO_BIN_EXE_otd"));
    let var = format!("OVERTHEDIFF_{}_CONFIG_DIR", env!("CARGO_PKG_VERSION_MAJOR"));
    command.arg("No such session").env(var, settings.path()).current_dir(cwd.path()).stdin(Stdio::null());
    let output = with_deadline(Duration::from_secs(60), "otd <unknown name> exits by itself", move || command.output().expect("run otd"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(100), "{stderr}");
    assert!(stderr.contains("No such session") && stderr.contains("Usage: otd"), "names the session and gives the usage: {stderr}");
}
