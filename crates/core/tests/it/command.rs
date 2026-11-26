//! Running commands by id with typed arguments, without a window (E01-S1-T12). This binary links
//! otd-core only, so no window exists: the context is a plain value.

use otd_core::command::{self, Args, Command, CommandError, Kind, Outcome, Param, State, Value};

/// A test context: a running total.
#[derive(Default)]
struct Tally {
    total: f64,
    locked: bool,
    shown: bool,
}

const ADD_PARAMS: &[Param] = &[
    Param::required("amount", Kind::Number { min: 0.0, max: 100.0 }),
    Param::optional("mode", Kind::Choice(&["add", "subtract"])),
    Param::optional("note", Kind::Text),
];

/// Adds or subtracts `amount` and returns the new total.
fn add(tally: &mut Tally, args: &Args) -> Outcome {
    let amount = args.number("amount").unwrap_or_default();
    tally.total += if args.text("mode") == Some("subtract") { -amount } else { amount };
    Ok(Some(Value::Number(tally.total)))
}

fn commands() -> Vec<Command<Tally>> {
    vec![
        Command::typed("test.add", "Add", ADD_PARAMS, add),
        Command::new("test.show", "Show Total", |tally: &mut Tally| tally.shown = true).needs_window(),
        Command::new("test.locked", "Reset", |tally: &mut Tally| tally.total = 0.0).state(|tally: &Tally| State::enabled(!tally.locked)),
    ]
}

#[test]
fn a_command_run_by_id_with_valid_arguments_returns_its_result_without_a_window() {
    let mut tally = Tally::default();
    let added = command::run(&commands(), &mut tally, "test.add", &[("amount", Value::Number(5.0))], false);
    assert_eq!(added, Ok(Some(Value::Number(tally.total))), "the result is the command's value");
    assert!(tally.total > 0.0, "it ran");

    let before = tally.total;
    let args = [("amount", Value::Number(2.0)), ("mode", Value::Choice("subtract".into())), ("note", Value::Text("x".into()))];
    let subtracted = command::run(&commands(), &mut tally, "test.add", &args, false);
    assert_eq!(subtracted, Ok(Some(Value::Number(tally.total))));
    assert!(tally.total < before, "the optional arguments reach the command");
}

#[test]
fn missing_unknown_or_out_of_range_arguments_are_errors_naming_the_argument_and_nothing_runs() {
    let number = |n: f64| ("amount", Value::Number(n));
    let cases: [(&[(&str, Value)], CommandError); 7] = [
        (&[], CommandError::MissingArgument("amount")),
        (&[number(1.0), ("colour", Value::Bool(true))], CommandError::UnknownArgument("colour".into())),
        (&[number(100.5)], CommandError::OutOfRange("amount")),
        (&[number(-0.5)], CommandError::OutOfRange("amount")),
        (&[number(f64::NAN)], CommandError::OutOfRange("amount")),
        (&[number(1.0), ("mode", Value::Choice("divide".into()))], CommandError::OutOfRange("mode")),
        (&[("amount", Value::Text("5".into()))], CommandError::WrongType("amount")),
    ];
    let mut tally = Tally::default();
    for (args, expected) in cases {
        let outcome = command::run(&commands(), &mut tally, "test.add", args, false);
        let message = outcome.as_ref().err().map(ToString::to_string).unwrap_or_default();
        assert_eq!(outcome, Err(expected.clone()), "for {args:?}");
        let name = expected.argument().unwrap_or("?");
        assert!(message.contains(name), "the message {message:?} names {name}");
    }
    let repeated = command::run(&commands(), &mut tally, "test.add", &[number(1.0), number(2.0)], false);
    assert_eq!(repeated, Err(CommandError::RepeatedArgument("amount")));
    assert_eq!(tally.total, 0.0, "no rejected call ran");
}

#[test]
fn a_command_that_needs_the_window_reports_that_when_run_headlessly() {
    let mut tally = Tally::default();
    let headless = command::run(&commands(), &mut tally, "test.show", &[], false);
    assert_eq!(headless, Err(CommandError::NeedsWindow("test.show")));
    assert!(!tally.shown, "it didn't claim success by running");
    assert!(headless.as_ref().err().map(ToString::to_string).is_some_and(|m| m.contains("test.show")));

    assert_eq!(command::run(&commands(), &mut tally, "test.show", &[], true), Ok(None), "with the window it runs");
    assert!(tally.shown);
}

#[test]
fn an_unknown_or_disabled_command_is_an_error_not_a_silent_success() {
    let mut tally = Tally { total: 3.0, locked: true, shown: false };
    assert_eq!(command::run(&commands(), &mut tally, "test.nothing", &[], true), Err(CommandError::UnknownCommand("test.nothing".into())));
    assert_eq!(command::run(&commands(), &mut tally, "test.locked", &[], true), Err(CommandError::Disabled("test.locked")));
    assert!(tally.total > 0.0, "the disabled command didn't run");
}
