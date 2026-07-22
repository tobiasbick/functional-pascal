# Phase 5 — Interactive terminal

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).
Phase 5 may change Tui3 and reuse existing `Std.Console` public operations. It must not add direct
`crossterm` access to Tui3, reuse `crates/fpas-vm/src/vm/execute/io/tui/bridge/`, introduce Tui3
widget intrinsics, or change the production `Std.Tui` session.

## Gate 5.A — Confirm the terminal boundary

**Status:** complete.

**Prerequisite:** Phase 4 (complete).

Audited the implemented Console boundary. The exact symbols and file-level ownership are recorded
in [runtime-boundary.md](../runtime-boundary.md); no additional compiler or VM behavior is needed.

## Task 5.1 — Interactive host acquisition and rollback

**Status:** complete.

Add focused Tui3 runtime modules for interactive session ownership and rollback; keep
`Runtime/Application.fpas` as orchestration. Implement the reverse-order cleanup state machine in
[runtime-boundary.md](../runtime-boundary.md). Add Rust failure-injection tests at the actual
terminal ownership layer and FPAS double-open/close diagnostics where headless verification is
possible.

**Done:** `Runtime/TerminalSession.fpas` pairs interactive ownership with host closing; the existing
Console coverage verifies transactional rollback and the double-acquire diagnostic. Headless hosts
remain independent and terminal-free.

## Task 5.2 — Map Console events to host input

**Status:** complete.

Add one focused event-adapter module. Map `Std.Console.Event` to normalized key, pointer, resize, and
tick inputs only; widget selection remains in FPAS routing. Add adapter tests for named keys,
modifiers, zero-based pointer conversion, resize ordering, and unsupported events.

## Task 5.3 — Flush the working surface

**Status:** complete.

Add one renderer module that converts the current Tui cell surface to the audited `Std.Console`
frame API. Color fallback occurs only here. Add headless renderer tests for styles, wide glyphs,
continuations, cursor policy, and resized surfaces; ordinary frames must not call
`SurfaceSnapshot`.

## Task 5.4 — Add `Run` and one interactive example

**Status:** complete.

Implement `TuiApplication.Run` with the same ordering as `RunIterations`; add one small example
under `examples/pascal/tui3/` using only model/update/view. The example must exercise resize,
keyboard, pointer, controlled input, one modal dialog, MenuBar, StatusLine, and Quit. Add a
non-interactive regression for the same model/update/view flow.

## Phase checkpoint

Run the common phase checkpoint plus the terminal restoration checks named by Gate 5.A. Record the
manual example command and result. Do not claim production readiness yet.

Automated checkpoint: `cargo fmt --check`, `cargo build --workspace`, FPAS formatting, and
`cargo run -q -p fpas-cli -- test --std-lib lib tests/` passed (445 passed, 1 skipped). The focused
Console ownership tests also passed. `cargo test --workspace` exceeded the 120-second local command
limit without reporting a failure. Manual terminal command:
`cargo run -q -p fpas-cli -- run --std-lib lib examples/pascal/tui3/interactive_demo.fpas`; it is
pending a real interactive TTY, so this phase does not claim production readiness.
