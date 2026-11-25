# E28 — Automation and agent access

**Group**: feature
**Label**: `area:automation`
**Size**: M
**Depends on**: E01 (command registry, installable builds)
**Source**: AGENTS.md rules 14, 15 and 18. Not a catalogue feature. The product command line (E11) and scripting (E20) are product features; this epic is the suite's automation interface.

This epic makes everything OverTheDiff can do available without the window. A command-line tool runs the registered commands on inputs given to it, and agents reach the same commands over MCP, limited to the folders they're given. New commands appear in both without extra work, because both are generated from E01-S1's command registry.

## Behavioral contract
- **Data**:
  - *Command call*: command id, typed arguments (JSON), and the result or error (JSON).
  - *Automation roots*: one read root and one write root, given at start-up.
- **Routes/endpoints**:
  - `otd-cli` (working name; the final executable name follows E01-S2-T1). It opens a session on the inputs given, runs commands, saves and reports.
  - `otd-cli mcp`: an MCP server over stdio.
- **Integrations**: E01-S1's command registry and session types; craft-mcp-server.
- **Rules/states**:
  - The tool reads and writes only the paths on its command line, or inside the automation roots for MCP. Paths are checked after resolving symbolic links and `..`.
  - A command that needs the window, or one that is partial or unsupported, says so in its result. It never reports success (AGENTS.md rule 15).
  - Exit codes distinguish success, a command error and a usage error. They are separate from the product's comparison exit codes (E11).

---

## Stories

### E28-S1 — Every command runs without the window
**Priority**: Should · **Layer**: 2 · **Size**: M · **Platforms**: All
**Needs**: E01-S1 (command registry), E01-S2 (installable builds)
**Traces**: AGENTS.md rules 14, 15
**Not in this story**: product switches and comparison exit codes → E11; script files → E20-S1; MCP → E28-S2.

As an automation author, I can open a comparison and run OverTheDiff's commands from a terminal without opening the app, so that whatever the app does can also run in a pipeline.

**Acceptance criteria**
- [ ] `otd-cli` opens a session of any registered type on inputs given on its command line, runs registered commands by id with JSON arguments, saves, and prints results and errors as JSON.
- [ ] Exit codes distinguish success, a command error and a usage error.
- [ ] It reads and writes only the paths named on its command line.
- [ ] A command that needs the window reports that it is unavailable without the window.
- [ ] `otd-cli` ships in every release package and runs from a terminal after installing.

#### Tasks
- **E28-S1-T1** — `otd-cli`: open sessions, run commands, save, report JSON · size: M · depends: — · flags: `naming-check`
  - AC: an integration test opens a Text Compare session on two temporary files, runs a copy-to-right command and saves, and the right file then matches. `--help` documents the syntax. Writing a path not named on the command line is refused with an error.
- **E28-S1-T2** — Ship `otd-cli` in the release packages, in its own pull request · size: S · depends: E28-S1-T1
  - AC: release packages on all three platforms include `otd-cli`, and it runs from a terminal after installing.

### E28-S2 — Agents work through MCP, limited to given folders
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E28-S1 (`otd-cli`)
**Traces**: AGENTS.md rule 18

As an AI agent, I can compare and reconcile files and folders through MCP, limited to the folders my operator gave me, so that I can use OverTheDiff safely on someone's machine.

**Acceptance criteria**
- [ ] `otd-cli mcp` refuses to start without both `--automation-read-root` and `--automation-write-root`.
- [ ] Every file access is checked against those roots after resolving symbolic links and `..`. Access outside them returns a tool error.
- [ ] MCP tools are generated from the command registry, so new commands appear without extra work.

#### Tasks
- **E28-S2-T1** — `otd-cli mcp` over stdio with folder limits, using craft-mcp-server · size: M · depends: — · flags: `craft-libs`
  - AC: a test runs initialize, tools/list and tools/call. A call that reads outside the read root, or writes outside the write root, including through a symbolic link, returns a tool error. A hostile-input sweep covers malformed messages.
