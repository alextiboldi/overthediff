//! Headless quick compare (E11-S2): `-qc=<type> <left> <right>`, also spelt `-quickcompare`, and
//! with `/` on Windows. It runs in the launching process, so it never opens a window or reaches a
//! running instance, and it claims the whole command line.
//!
//! *Unspecified*, decided here: the switch name and type match case-sensitively, and any other
//! switch alongside it is an error.

use otd_core::i18n::{fmt_plain, t};
use otd_launch::{Completion, Handler, Mode, Pattern, Request, RunHere};

use crate::Shell;

/// Exit code of a quick compare that couldn't run: 100, Unknown error (E11 behavioral contract).
const EXIT_ERROR: i32 = 100;

/// What a quick compare tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Test {
    Size,
    Crc,
    Binary,
}

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
        match parse(&request.args).and_then(|(test, files)| run(test, files)) {
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

/// The exit code of comparing `files` by `test`.
// ponytail: E11-S2-T2 runs the comparison; until then a valid command line says so and fails.
fn run(test: Test, _files: [&str; 2]) -> Result<i32, String> {
    Err(fmt_plain(t("Quick compare by {type} isn't built yet."), &[("type", test.name())]))
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
}
