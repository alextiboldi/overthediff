//! Headless quick compare (E11-S2): `-qc=<type> <left> <right>`, also spelt `-quickcompare`, and
//! with `/` on Windows. It runs in the launching process, so it never opens a window or reaches a
//! running instance, and it claims the whole command line.
//!
//! *Unspecified*, decided here: the switch name and type match case-sensitively, and any other
//! switch alongside it is an error.

use std::path::Path;

use otd_core::i18n::{fmt_plain, t};
use otd_folder_engine::{Aligned, ContentOptions, ContentTest, Criteria, DEFAULT_TOLERANCE, Status};
use otd_launch::{Completion, Handler, Mode, Pattern, Request, RunHere};
use otd_source::{DataSource as _, Item, ItemKind, Links, LocalSource};

use crate::Shell;

/// Exit code of a quick compare that couldn't run: 100, Unknown error (E11 behavioral contract).
const EXIT_ERROR: i32 = 100;
/// The files are the same by the chosen test: 1, Binary same (E11 behavioral contract).
const EXIT_SAME: i32 = 1;
/// The files differ by the chosen test: 11, Binary differences.
const EXIT_DIFFERENT: i32 = 11;

/// What a quick compare tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Test {
    Size,
    Crc,
    Binary,
}

#[cfg(test)]
impl Test {
    fn name(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::Crc => "crc",
            Self::Binary => "binary",
        }
    }
}

/// The quick-compare handler under one of its switch names, `qc` or `quickcompare`.
pub struct QuickCompare(pub &'static str);

impl Handler<Shell> for QuickCompare {
    fn id(&self) -> &'static str {
        self.0
    }

    fn pattern(&self) -> Pattern {
        Pattern::Switch(self.0)
    }

    fn mode(&self) -> Mode {
        Mode::RunHere
    }

    fn usage(&self) -> &'static str {
        match (self.0, cfg!(windows)) {
            ("qc", true) => "/qc=<size|crc|binary> <left> <right>   Compare two files without a window: exit 1 if same, 11 if different",
            ("qc", false) => "-qc=<size|crc|binary> <left> <right>   Compare two files without a window: exit 1 if same, 11 if different",
            (_, true) => "/quickcompare=<size|crc|binary> <left> <right>   The same as /qc",
            (_, false) => "-quickcompare=<size|crc|binary> <left> <right>   The same as -qc",
        }
    }

    fn run_here(&self, request: &Request) -> RunHere {
        match parse(&request.args).and_then(|(test, files)| run(test, files, &request.cwd)) {
            Ok(code) => RunHere::Exit(code),
            Err(message) => {
                eprintln!("otd: {message}");
                RunHere::Exit(EXIT_ERROR)
            }
        }
    }

    /// Never reached: [`Mode::RunHere`] and a `run_here` that always exits hand nothing off.
    fn open(&self, _request: Request, _done: Completion, _shell: &mut Shell) {}
}

/// The test and the two files a quick-compare command line asks for, or the one-line error.
fn parse(args: &[String]) -> Result<(Test, [&str; 2]), String> {
    let mut test = None;
    let mut files = Vec::new();
    for arg in args {
        match otd_launch::switch(arg) {
            None => files.push(arg.as_str()),
            Some(("qc" | "quickcompare", value)) if test.is_none() => test = Some(parse_test(arg, value)?),
            Some(_) => return Err(fmt_plain(t("{switch} can't be used with quick compare."), &[("switch", arg)])),
        }
    }
    let test = test.ok_or_else(|| t("Quick compare needs -qc=<type>.").to_owned())?;
    let &[left, right] = files.as_slice() else {
        return Err(fmt_plain(t("Quick compare needs two files, not {count}."), &[("count", &files.len().to_string())]));
    };
    Ok((test, [left, right]))
}

/// `switch` is the whole argument, as typed.
fn parse_test(switch: &str, value: Option<&str>) -> Result<Test, String> {
    match value {
        Some("size") => Ok(Test::Size),
        Some("crc") => Ok(Test::Crc),
        Some("binary") => Ok(Test::Binary),
        // A quick compare by the session's rules is E08-S13's.
        None => Err(fmt_plain(
            t("{switch} without a type, comparing by rules, isn't supported yet; use {switch}=size, crc or binary."),
            &[("switch", switch)],
        )),
        Some(other) => Err(fmt_plain(t("{type} isn't a quick-compare type; use size, crc or binary."), &[("type", other)])),
    }
}

/// The exit code of comparing `files`, relative to `cwd`, by `test`: 1 when they are the same,
/// 11 when they differ.
fn run(test: Test, files: [&str; 2], cwd: &Path) -> Result<i32, String> {
    let [(left, left_item), (right, right_item)] = [open(files[0], cwd)?, open(files[1], cwd)?];
    let row = Aligned { name: left_item.name.clone(), left: Some(left_item), right: Some(right_item) };
    // Size runs only the quick size test; crc and binary only their content test.
    let criteria = Criteria {
        size: test == Test::Size,
        modified: false,
        tolerance: DEFAULT_TOLERANCE,
        filename_case: false,
        unix_permissions: false,
        unix_owner: false,
        unix_group: false,
    };
    let content = match test {
        Test::Size => None,
        Test::Crc => Some(ContentTest::Crc),
        Test::Binary => Some(ContentTest::Binary),
    };
    let options = ContentOptions { test: content, skip_if_quick_same: false, override_quick_results: true };
    let here = Path::new("");
    match otd_folder_engine::compare(&row, &criteria, &options, (&left, here), (&right, here)).status {
        Status::Same => Ok(EXIT_SAME),
        Status::Error(error) => Err(error.message),
        // Two files with no time test are only ever same, different or an error.
        _ => Ok(EXIT_DIFFERENT),
    }
}

/// A source on the folder holding the file `spec` names, and the file as that source reports it.
/// `spec` is any path the file-system access layer accepts (`~/…`, `smb://…`, a UNC or OS path),
/// relative to `cwd`. A missing, unreadable or non-file item is an error.
fn open(spec: &str, cwd: &Path) -> Result<(LocalSource, Item), String> {
    let not_a_file = || fmt_plain(t("{file} isn't a file."), &[("file", spec)]);
    let path = cwd.join(otd_source::resolve(spec).map_err(|e| e.message)?);
    let (Some(folder), Some(name)) = (path.parent(), path.file_name()) else { return Err(not_a_file()) };
    let source = LocalSource::new(folder);
    let name = Path::new(name);
    let item = source.stat(name, Links::Follow).map_err(|e| e.message)?;
    if let Some(error) = item.error {
        return Err(error.message);
    }
    // otd-source counts pipes, sockets and devices as files; opening a pipe would block forever.
    if item.kind != ItemKind::File || !std::fs::metadata(&path).is_ok_and(|meta| meta.is_file()) {
        return Err(not_a_file());
    }
    // The size test reads no content, so an unreadable file would otherwise pass it.
    drop(source.open_read(name).map_err(|e| e.message)?);
    Ok((source, item))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREFIX: &str = if cfg!(windows) { "/" } else { "-" };

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.replacen('-', PREFIX, 1)).collect()
    }

    #[test]
    fn every_spelling_of_every_type_is_claimed_to_run_here_and_parsed() {
        let registry = crate::handlers::registry();
        for name in ["qc", "quickcompare"] {
            for test in [Test::Size, Test::Crc, Test::Binary] {
                let args = args(&[&format!("-{name}={}", test.name()), "left.bin", "right.bin"]);
                let handler = registry.claim(&args).ok().flatten();
                assert!(handler.is_some_and(|h| h.id() == name && h.mode() == Mode::RunHere), "{args:?} is claimed by {name} to run here");
                assert_eq!(parse(&args), Ok((test, ["left.bin", "right.bin"])), "{args:?}");
            }
        }
    }

    #[test]
    fn a_command_line_that_is_not_one_type_and_two_files_is_refused() {
        for list in [
            &["-qc=bytes", "a", "b"][..],
            &["-qc", "a", "b"],
            &["-qc=", "a", "b"],
            &["-qc=size", "a"],
            &["-qc=size", "a", "b", "c"],
            &["-qc=size", "-title1=x", "a", "b"],
            &["-qc=size", "-quickcompare=crc", "a", "b"],
        ] {
            assert!(parse(&args(list)).is_err(), "{list:?} is refused");
        }
    }

    #[test]
    fn an_unknown_type_exits_with_100() {
        let request = Request { handler: Some("qc".into()), args: args(&["-qc=bytes", "a", "b"]), cwd: ".".into(), attachment: None };
        assert_eq!(QuickCompare("qc").run_here(&request), RunHere::Exit(EXIT_ERROR));
    }

    /// The exit code of `-qc=<test> <left> <right>` run in `dir`, so the names are relative to it.
    fn code(test: &str, left: &str, right: &str, dir: &Path) -> RunHere {
        let request = Request { handler: Some("qc".into()), args: args(&[&format!("-qc={test}"), left, right]), cwd: dir.into(), attachment: None };
        QuickCompare("qc").run_here(&request)
    }

    fn files(list: &[(&str, &[u8])]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temporary folder");
        for (name, content) in list {
            std::fs::write(dir.path().join(name), content).expect("write a test file");
        }
        dir
    }

    #[test]
    fn a_missing_file_or_a_folder_exits_with_100_for_every_test() {
        let dir = files(&[("a", b"x")]);
        std::fs::create_dir(dir.path().join("folder")).expect("create a test folder");
        for test in ["size", "crc", "binary"] {
            for (left, right) in [("a", "missing"), ("missing", "a"), ("a", "folder"), ("folder", "folder"), ("a", "..")] {
                assert_eq!(code(test, left, right, dir.path()), RunHere::Exit(EXIT_ERROR), "{test} {left} {right}");
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_device_exits_with_100_for_every_test() {
        let dir = files(&[("a", b"")]);
        for test in ["size", "crc", "binary"] {
            assert_eq!(code(test, "a", "/dev/null", dir.path()), RunHere::Exit(EXIT_ERROR), "{test}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_exits_with_100_even_by_size() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = files(&[("a", b"x"), ("locked", b"x")]);
        let locked = dir.path().join("locked");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).expect("lock the test file");
        if std::fs::File::open(&locked).is_ok() {
            // Straight to stderr, past the test harness's capture, so a passing run still lists it.
            use std::io::Write as _;
            let _ = writeln!(std::io::stderr(), "skipped the unreadable-file case: this account reads files whatever their permissions");
        } else {
            assert_eq!(code("size", "a", "locked", dir.path()), RunHere::Exit(EXIT_ERROR));
        }
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o600)).expect("unlock the test file");
    }
}
