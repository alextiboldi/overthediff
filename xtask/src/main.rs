//! Workspace tooling: `cargo xtask <command>`. Standard library only; it runs `cargo` as a
//! subprocess.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  ci       fmt --check, clippy -D warnings, test, ui-free (stops at the first failure)
  ui-free  check that no crate under crates/, except crates/ui and crates/ui-*, pulls in egui, eframe or winit
";

/// Package-name prefixes that mark a UI dependency. Crates under `crates/`, except the UI crates
/// `crates/ui` and `crates/ui-*`, must not reach any of them through normal dependencies, so engines,
/// data sources and pattern services test headlessly.
const UI_PREFIXES: [&str; 5] = ["egui", "eframe", "epaint", "emath", "winit"];

fn main() -> ExitCode {
    let result = match std::env::args().nth(1).as_deref() {
        Some("ci") => ci(),
        Some("ui-free") => ui_free(),
        Some("-h" | "--help" | "help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root: the parent of this crate's directory.
fn root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap_or(manifest).to_path_buf()
}

fn cargo(args: &[&str]) -> Command {
    let mut c = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    c.current_dir(root()).args(args);
    c
}

fn run(args: &[&str]) -> Result<(), String> {
    let what = format!("cargo {}", args.join(" "));
    eprintln!("$ {what}");
    let status = cargo(args).status().map_err(|e| format!("{what}: failed to start: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("{what}: exited with {status}")) }
}

fn ci() -> Result<(), String> {
    run(&["fmt", "--all", "--", "--check"])?;
    run(&["clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"])?;
    run(&["test", "--workspace", "--locked"])?;
    ui_free()?;
    eprintln!("ci: all checks passed");
    Ok(())
}

/// Fails if any crate under `crates/`, except `crates/ui` and `crates/ui-*`, has a UI package in its
/// normal dependency tree, on any target, with all its features on (an optional UI dependency is
/// still a UI dependency, and workspace feature unification can switch it on).
fn ui_free() -> Result<(), String> {
    let dir = root().join("crates");
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut offenders = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        let manifest = path.join("Cargo.toml");
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if name == "ui" || name.starts_with("ui-") || !manifest.is_file() {
            continue;
        }
        let manifest = manifest.to_string_lossy().into_owned();
        let args =
            ["tree", "--manifest-path", &manifest, "--edges", "normal", "--target", "all", "--all-features", "--prefix", "none", "--format", "{p}"];
        let out = cargo(&args).output().map_err(|e| format!("cargo tree: failed to start: {e}"))?;
        if !out.status.success() {
            return Err(format!("cargo tree --manifest-path {manifest} failed:\n{}", String::from_utf8_lossy(&out.stderr)));
        }
        let tree = String::from_utf8_lossy(&out.stdout);
        let mut lines = tree.lines();
        let krate = lines.next().unwrap_or_default().to_owned();
        let ui: Vec<&str> = lines.filter(|l| UI_PREFIXES.iter().any(|p| l.starts_with(p))).collect();
        if !ui.is_empty() {
            offenders.push(format!("{krate} depends on {}", ui.join(", ")));
        }
    }
    if offenders.is_empty() {
        eprintln!("ui-free: no crate under crates/, except crates/ui and crates/ui-*, depends on a UI package");
        Ok(())
    } else {
        Err(format!("UI dependencies in crates/ (UI code belongs in apps/, crates/ui or crates/ui-*):\n  {}", offenders.join("\n  ")))
    }
}
