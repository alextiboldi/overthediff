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
- **E27-S1-T1** — Cargo workspace with a UI-free library crate and the app crate · size: S · depends: —
  - AC: the workspace builds. The library crate's manifest has no egui or other UI dependency. AGENTS.md `## This repository` names the crates, the checks, and where commands, colour roles and tests live.
- **E27-S1-T2** — CI through craft-common-actions, in its own pull request · size: S · depends: E27-S1-T1
  - AC: a pull request runs workspace-wide format, clippy and test checks on Linux and macOS, and Windows runs on `main`. A new crate is covered without changing the workflow.
- **E27-S1-T3** — Test layout, a keyboard-and-accessibility UI harness, and a hostile-input sweep · size: M · depends: E27-S1-T1 · flags: `accessibility`, `licence-check`
  - AC: each crate with integration tests has one `tests/it/main.rs` binary. A UI test opens a test command by its shortcut, and after each step it checks where focus is and the focused node's role and name. A hostile-input sweep runs a sample parser over truncated and damaged input without a panic or a hang.
