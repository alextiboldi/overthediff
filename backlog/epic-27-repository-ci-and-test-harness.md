# E27 — Repository, CI and test harness

**Group**: foundation
**Label**: `area:repo`
**Size**: M
**Depends on**: —
**Source**: AGENTS.md (rules 13, 17, 21, 27–33) and the suite standards it links. Not a catalogue feature: this is the groundwork every story builds on.

This epic sets up the repository so several people and agents can work at once. It covers a Cargo workspace whose engines don't depend on the UI, CI through the shared workflows, and test shapes that every later story copies: integration tests, a UI harness that drives the keyboard and reads the accessibility tree, and hostile-input sweeps.

## Behavioral contract
- **Data**: none for users. The workspace layout, crate names and checks are recorded in AGENTS.md `## This repository`.
- **Routes/endpoints**: none for users. The checks are `cargo fmt`, `cargo clippy` and `cargo test` across the workspace.
- **Integrations**:
  - storytold/craft-common-actions, for CI.
  - egui_kittest, for UI tests.
  - craft-testkit's `hostile` module, for hostile-input sweeps.
- **Rules/states**:
  - CI changes go in pull requests of their own (AGENTS.md rule 17).
  - Engine, data-source and pattern code lives in library crates with no UI dependency, so the E03 and E04 suites run headlessly.
  - Lints are fixed, not silenced (rule 13).

---

## Stories

### E27-S1 — A repository several people can work on at once
**Priority**: Must · **Layer**: setup · **Size**: M · **Platforms**: All
**Needs**: —
**Traces**: AGENTS.md rules 13, 17, 21, 30, 33
**Not in this story**: packaging → E01-S2; the command-line automation tool → E28-S1.

As a contributor, I can work on one feature without colliding with every other open pull request. I can also test interface behaviour without a person at the keyboard.

**Acceptance criteria**
- [ ] The repository is a Cargo workspace. Engines, data sources and pattern services live in library crates that don't depend on the UI, separate from the app crate.
- [ ] CI runs format, lint and test checks on every pull request through storytold/craft-common-actions.
- [ ] UI tests drive the app by keyboard and read its accessibility tree.
- [ ] AGENTS.md `## This repository` describes the layout, the checks and how to run each kind of test.

#### Tasks
- **E27-S1-T1** — Cargo workspace with a UI-free library crate and the app crate · size: S · depends: — · status: needs-verification
  - AC: the workspace builds. The library crate's manifest has no egui or other UI dependency. AGENTS.md `## This repository` names the crates, the checks, and where commands, colour roles and tests live.
  - Implemented: a Cargo workspace with member globs `apps/*` and `crates/*`, the app crate `apps/otd` (executable `otd`, a minimal entry point that E01-S1 replaces), the UI-free library crate `crates/core` (`otd-core`), and `xtask` (`cargo xtask ci`: fmt check, clippy with warnings denied, workspace tests, and the `ui-free` dependency check); workspace lints, `rustfmt.toml`, `clippy.toml`, the egui and winit `[patch.crates-io]` forks, the recorded craft-libs rev for later crates, NOTICE and ATTRIBUTION.md; AGENTS.md `## This repository` names the crates, pins, checks and where commands, colour roles and tests live. `cargo xtask ci` passed on macOS; no PR CI exists until E27-S1-T2. https://github.com/alextiboldi/overthediff/pull/1
  - Verify by hand: once E27-S1-T2 is on `main`, confirm the workspace builds and `cargo xtask ci` passes on the Linux and Windows CI legs.
- **E27-S1-T2** — CI through craft-common-actions, in its own pull request · size: S · depends: E27-S1-T1 · status: needs-verification
  - AC: a pull request runs workspace-wide format, clippy and test checks on Linux and macOS, and Windows runs on `main`. A new crate is covered without changing the workflow.
  - Implemented: `.github/workflows/ci.yml` calls storytold/craft-common-actions `gate.yml@main`, which runs `cargo xtask ci` (fmt, clippy with warnings denied, workspace tests, the `ui-free` check) across the whole workspace, so new crates under `apps/` or `crates/` are covered without editing it. Pull requests run `ubuntu-24.04` and `macos-latest`; pushes to `main` and manual runs add `windows-latest`. Linux installs the Wayland and X11 build packages; AGENTS.md `## This repository` names the workflow. The pull request's own run passed on Linux and macOS. https://github.com/alextiboldi/overthediff/pull/2
  - Verify by hand: after merge, the first push to `main` shows a `gate (windows-latest)` check in its CI run and it passes; this also covers E27-S1-T1's Linux and Windows check.
- **E27-S1-T3** — Test layout, a keyboard-and-accessibility UI harness, and a hostile-input sweep · size: M · depends: E27-S1-T1 · flags: `accessibility`, `licence-check`
  - AC: each crate with integration tests has one `tests/it/main.rs` binary. A UI test opens a test command by its shortcut, and after each step it checks where focus is and the focused node's role and name. A hostile-input sweep runs a sample parser over truncated and damaged input without a panic or a hang.
