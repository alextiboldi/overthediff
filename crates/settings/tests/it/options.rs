//! The Options dialog's values in the settings store: each value type reads back after a restart,
//! and a saved value that doesn't fit its option reads as the option's default.

use std::collections::BTreeMap;

use otd_core::options::{OptionSpec, Value};
use otd_settings::Location;

fn options() -> Vec<OptionSpec> {
    let option = |id, default| OptionSpec { page: &["Page"], id, label: id, default, choices: &[] };
    vec![
        option("bool", Value::Bool(false)),
        option("number", Value::Number(4.0)),
        OptionSpec { choices: &["Lines", "Words"], ..option("choice", Value::Choice("Lines".into())) },
        option("text", Value::Text("Mono".into())),
        option("path", Value::Path(String::new())),
        option("colour", Value::Colour([0, 0, 0, 255])),
    ]
}

fn defaults() -> BTreeMap<String, Value> {
    options().into_iter().map(|o| (o.id.to_owned(), o.default)).collect()
}

fn read(dir: &std::path::Path) -> BTreeMap<String, Value> {
    let (mut store, _) = otd_settings::open(Location::Dir(dir.to_path_buf()));
    otd_settings::options::read(&mut store, &options())
}

#[test]
fn every_value_type_reads_back_after_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let changed: BTreeMap<String, Value> = [
        ("bool", Value::Bool(true)),
        ("number", Value::Number(-2.5)),
        ("choice", Value::Choice("Words".into())),
        ("text", Value::Text("Fira \"Code\"".into())),
        ("path", Value::Path("C:\\dir\\a b.txt".into())),
        ("colour", Value::Colour([0xAB, 0x0C, 0xFF, 0x80])),
    ]
    .into_iter()
    .map(|(id, v)| (id.to_owned(), v))
    .collect();
    assert!(changed.iter().all(|(id, v)| defaults()[id] != *v), "every value differs from its default");
    let (mut store, _) = otd_settings::open(Location::Dir(dir.path().to_path_buf()));
    otd_settings::options::write(&mut store, &changed);
    store.flush().unwrap();
    drop(store);
    assert_eq!(read(dir.path()), changed);
}

#[test]
fn a_saved_value_that_doesnt_fit_its_option_reads_as_its_default_and_leaves_the_others() {
    let dir = tempfile::tempdir().unwrap();
    let file = "schema_version = 1\n\n[options]\nbool = \"yes\"\nnumber = nan\nchoice = \"Characters\"\ntext = 3\npath = \"kept.txt\"\ncolour = \"#12345\"\n";
    std::fs::write(dir.path().join("config.toml"), file).unwrap();
    let mut expected = defaults();
    expected.insert("path".into(), Value::Path("kept.txt".into()));
    assert_eq!(read(dir.path()), expected);
}
