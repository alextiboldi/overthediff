//! Embeds the craft-i18n catalogs (storytold/craft-i18n) as `CRAFT_I18N` in `OUT_DIR/craft_i18n.rs`,
//! following craft-i18n's `docs/integration.md`: one (locale, `common.tsv`, `overthediff.tsv`)
//! entry per locale under `$CRAFT_I18N_DIR/locales` that has `overthediff.tsv`. With
//! `CRAFT_I18N_DIR` unset, or not a checkout (a warning), the list is empty and the app is English
//! only; with `CRAFT_I18N_REQUIRED=1` (releases) that is an error instead.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const APP_CATALOG: &str = "overthediff.tsv";

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_REQUIRED");
    let required = std::env::var("CRAFT_I18N_REQUIRED").is_ok_and(|v| v.trim() == "1");
    let level = if required { "error" } else { "warning" };
    let locales = match std::env::var_os("CRAFT_I18N_DIR").filter(|dir| !dir.is_empty()) {
        // Not canonicalized: on Windows that gives `\\?\C:\…`, which `include_str!` can't read.
        Some(dir) => locales(&PathBuf::from(dir).join("locales")).unwrap_or_else(|err| {
            println!("cargo::{level}=CRAFT_I18N_DIR: {err}; the interface is English only");
            Vec::new()
        }),
        None => {
            if required {
                println!("cargo::error=CRAFT_I18N_REQUIRED=1 but CRAFT_I18N_DIR is unset");
            }
            Vec::new()
        }
    };
    let mut out = String::from("/// (locale, shared catalog, OverTheDiff's catalog), one entry per locale.\n");
    out.push_str("static CRAFT_I18N: &craft_i18n::Embedded = &[\n");
    for (code, shared, app) in locales {
        // `{:?}` writes the path as a Rust string literal, escaping Windows backslashes.
        let shared = shared.map_or_else(|| "\"\"".to_owned(), |path| format!("include_str!({path:?})"));
        let _ = writeln!(out, "    ({code:?}, {shared}, include_str!({app:?})),");
    }
    out.push_str("];\n");
    let path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("craft_i18n.rs");
    if let Err(err) = std::fs::write(&path, out) {
        println!("cargo::error=writing {}: {err}", path.display());
    }
}

/// Each locale under `dir` that has [`APP_CATALOG`]: its code, its `common.tsv` if any, and its app
/// catalog, sorted by code.
fn locales(dir: &Path) -> Result<Vec<(String, Option<PathBuf>, PathBuf)>, String> {
    // `include_str!` resolves a relative path against the generated file, not this crate.
    if !dir.is_absolute() {
        return Err(format!("{} is not an absolute path", dir.display()));
    }
    let entries = std::fs::read_dir(dir).map_err(|err| format!("{} is not a craft-i18n checkout ({err})", dir.display()))?;
    println!("cargo::rerun-if-changed={}", dir.display());
    let mut locales = Vec::new();
    for path in entries.flatten().map(|entry| entry.path()) {
        let (Some(code), app) = (path.file_name().and_then(|name| name.to_str()).map(str::to_owned), path.join(APP_CATALOG)) else { continue };
        if !app.is_file() {
            continue;
        }
        let shared = Some(path.join("common.tsv")).filter(|shared| shared.is_file());
        for file in shared.iter().chain([&app]) {
            println!("cargo::rerun-if-changed={}", file.display());
        }
        locales.push((code, shared, app));
    }
    if locales.is_empty() {
        return Err(format!("{} has no locale with an {APP_CATALOG}", dir.display()));
    }
    locales.sort();
    Ok(locales)
}
