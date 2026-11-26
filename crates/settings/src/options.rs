//! The Options dialog's values (E01-S1-T6), kept in the `options` section, one key per option id,
//! as plain TOML values: a boolean, a number, a string, or a colour as `"#rrggbbaa"`.
//!
//! Each value is read against its option: one of the wrong type, a choice the option doesn't
//! offer or a number that isn't finite reads as the option's default, and leaves the other
//! options alone.

use std::collections::BTreeMap;

use otd_core::options::{OptionSpec, Value};

use crate::Store;

/// The section the values live in.
pub const SECTION: &str = "options";

/// Each option's saved value, or its default when there is none or it doesn't fit.
pub fn read(store: &mut Store, options: &[OptionSpec]) -> BTreeMap<String, Value> {
    let saved: toml::Table = store.section(SECTION);
    options.iter().map(|o| (o.id.to_owned(), saved.get(o.id).and_then(|v| from_toml(o, v)).unwrap_or_else(|| o.default.clone()))).collect()
}

/// Saves `values`, shortly after (or on `flush`). Every option's value is written, its default
/// included, so a value set back to its default replaces the one in the file.
pub fn write(store: &mut Store, values: &BTreeMap<String, Value>) {
    let table: toml::Table = values.iter().map(|(id, v)| (id.clone(), to_toml(v))).collect();
    // A table of plain values always serializes.
    store.update(|s| s.set(SECTION, &table).unwrap_or(()));
}

fn to_toml(value: &Value) -> toml::Value {
    match value {
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::Number(n) => toml::Value::Float(*n),
        Value::Choice(s) | Value::Text(s) | Value::Path(s) => toml::Value::String(s.clone()),
        Value::Colour([r, g, b, a]) => toml::Value::String(format!("#{r:02x}{g:02x}{b:02x}{a:02x}")),
    }
}

/// `saved` as a value of `option`'s type, if it is one.
fn from_toml(option: &OptionSpec, saved: &toml::Value) -> Option<Value> {
    Some(match (&option.default, saved) {
        (Value::Bool(_), toml::Value::Boolean(b)) => Value::Bool(*b),
        (Value::Number(_), toml::Value::Float(n)) if n.is_finite() => Value::Number(*n),
        (Value::Number(_), toml::Value::Integer(n)) => Value::Number(*n as f64),
        (Value::Choice(_), toml::Value::String(s)) if option.choices.contains(&s.as_str()) => Value::Choice(s.clone()),
        (Value::Text(_), toml::Value::String(s)) => Value::Text(s.clone()),
        (Value::Path(_), toml::Value::String(s)) => Value::Path(s.clone()),
        (Value::Colour(_), toml::Value::String(s)) => Value::Colour(colour(s)?),
        _ => return None,
    })
}

/// `#rrggbbaa`, in hex digits of either case.
fn colour(text: &str) -> Option<[u8; 4]> {
    let hex = text.strip_prefix('#').filter(|h| h.len() == 8 && h.is_ascii())?;
    let mut rgba = [0; 4];
    for (i, byte) in rgba.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(rgba)
}
