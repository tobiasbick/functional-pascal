# TUI control server — architecture (proposal)

**Status:** design only — not implemented.

Host integration sketch for the HTTP API in [`control-server-api.md`](control-server-api.md).

## Problem statement

Hosted `Std.Tui` programs run a Rust-owned event loop on the main VM thread. External tools need:

1. **Observation** — screen content, view tree, focus, modal depth, captured stdout.
2. **Injection** — keyboard, mouse, resize, paste, and line input without modifying FPAS source.
3. **Independence** — read and write operations usable separately.

The design reuses proven test infrastructure instead of inventing a terminal emulator protocol.

## Placement in the stack

```text
 External client (curl, script, MCP)
              │
              ▼ HTTP/JSON (127.0.0.1)
 ┌────────────────────────────┐
 │  fpas-cli debug server     │  ← new: optional background thread
 │  (parse JSON → VM calls)   │
 └─────────────┬──────────────┘
               │
               ▼
 ┌────────────────────────────┐
 │  fpas-vm                   │
 │  SharedState mutexes       │
 │  • console (screen)        │
 │  • key_input (event queues)│
 │  • tui (views, host, …)   │
 └─────────────┬──────────────┘
               │
               ▼
 ┌────────────────────────────┐
 │  fpas-std                  │
 │  TuiSession, ViewRegistry  │
 │  UiHost, ConsoleState      │
 └────────────────────────────┘
```

This is **not** exposed through `Std.Tui` or FPAS intrinsics. It is a CLI / VM developer feature, similar to how `fpas test` applies sidecars before run but available **during** run.

## Why not FPAS standard library?

| Concern | Reason to keep out of `Std.*` |
| ------- | ----------------------------- |
| Security | User programs must not open control ports by default |
| Scope | Debugging and automation, not application logic |
| Policy | Matches [`docs/future/std-roadmap.md`](../std-roadmap.md): `Std.Net` / `Std.Http` are later candidates for app networking, not host introspection |

## Reuse from today's test path

| Today | Control server |
| ----- | -------------- |
| `<test>.script.toml` queued before `vm.run()` | Same event shapes, queued on demand via POST |
| `apply_script_to_vm` | Shared mapping from JSON → `ConsoleEvent` / `GraphEvent` |
| `*.expect.screen` after run | `GET /screen` at any time |
| Rust `tui_host_vm` tests calling `push_console_event` | Same queue, different caller |

Event injection should call the same helpers as [`test_script/console.rs`](../../../crates/fpas-cli/src/test_script/console.rs) to avoid two enum string tables.

## Threading model

**Proposed v1**

- Main thread: VM execution and TUI host loop (unchanged).
- Background thread: accept HTTP connections, parse requests, perform short critical sections on `SharedState`, respond.

**Read path**

1. Accept request on background thread.
2. Lock `console` or `tui` mutex (one at a time — see lock ordering below).
3. Copy serializable snapshot (strings, numbers, JSON tree).
4. Release lock, send response.

Reads must never block on the event loop completing a handler.

**Write path**

1. Parse JSON body.
2. Lock `key_input` (or graph input mutex for graph surface).
3. Push to `console_event_queue` / readln queue (same as tests).
4. Release lock, return `202 Accepted`.

The main loop picks up queued events on its next poll, identical to pre-run scripting.

**Optional `/control/step` (v2)**

Requires a channel or flag the main loop checks between `ProcessNext` iterations. More invasive; defer until fire-and-forget injection proves useful.

## Lock ordering

[`SharedState`](../../../crates/fpas-vm/src/vm/shared.rs) documents that VM code must not hold multiple I/O mutexes without a fixed order. The debug server must follow the same rule:

- Never hold `tui` and `console` simultaneously unless a single documented helper owns both briefly for a consistent snapshot (prefer separate endpoints).
- Write endpoints touch **only** `key_input` / graph queues.
- Read endpoints touch **one** of `console`, `tui`, or stdout capture at a time.

Keep critical sections small to avoid stalling the TUI loop.

## Operating modes

### Live terminal

User runs `fpas --debug-port 9333` in a real terminal. Injected events are consumed from the test/console queues **before** live crossterm polling when `test_mode` is active on the queue, matching [`KeyInput`](../../../crates/fpas-std/src/console/key_input/mod.rs) behavior.

Use case: develop the IDE, automate one action from a script while watching the terminal.

### Headless

No terminal writer attached (same as `fpas test`). All input comes from POST endpoints or pre-run scripts.

Use case: CI automation with stepwise assertions via external driver.

Both modes expose the same API; `GET /health` reports `headless`.

## CLI and environment

Proposed activation (exact names open):

| Mechanism | Example |
| --------- | ------- |
| CLI flag | `fpas run --debug-port 9333 <path>` |
| Environment | `FPAS_DEBUG_PORT=9333` |
| Disable | omit flag / unset env (default off) |

The server must **fail fast** if the port is already in use, with a clear message on stderr.

`fpas test` might gain `--debug-port` later for interactive debugging of a single test; not required for v1.

## Security

- Bind to `127.0.0.1` only; reject `0.0.0.0`.
- No authentication in v1 (local developer tool). Document that enabling the port allows any local process to inject input and request quit.
- Do not enable in release builds distributed to end users unless explicitly requested (optional `cfg` or feature flag).

## Relation to sidecars and golden files

| Mechanism | When | Deterministic |
| --------- | ---- | ------------- |
| `.script.toml` | Before run | Yes |
| `*.expect.screen` | After run | Yes |
| Control server | During run | Depends on timing unless `/control/step` exists |

Recommended workflow:

- **CI:** keep sidecars + golden files.
- **Local debug:** control server for exploration.
- **New integration tests:** optionally drive headless runs via HTTP from an external test harness; still assert with JSON snapshots instead of golden files when flakiness is a concern.

## Implementation layout (when started)

Proposed crate boundaries (subject to change):

```text
crates/fpas-cli/src/debug_server/
 ├── mod.rs           — startup/shutdown, port config
 ├── http.rs          — minimal HTTP router (or thin hyper/axum wrapper)
 ├── read.rs          — GET handlers → VM snapshots
 ├── write.rs         — POST handlers → push_* calls
 └── serialize.rs     — ViewRegistry → JSON
```

Alternative: small `fpas-debug` crate if the CLI grows too large. Prefer CLI-owned first per AGENTS.md module sizing.

Wire startup from [`cli_run`](../../../crates/fpas-cli/src/cli_run/) when debug port is set, after VM construction and before `vm.run()`.

## Graph applications

`Std.Graph` hosted apps use a parallel input path (`Vm::push_graph_event`). Options:

1. **Same port, `surface` field** — `GET /health` reports `"graph"`; graph-specific POST routes under `/events/graph/*`.
2. **Separate default port** — simpler v1 if TUI-only.

Recommendation: design the API with a `surface` discriminator from the start; implement TUI routes first.

## Open decisions

1. **HTTP library** — `hyper` / `axum` vs minimal std-only parser. Favor a small maintained dependency if it reduces bug surface.
2. **`/control/step` in v1?** — Defer unless a concrete test needs it immediately.
3. **WebSocket push** — optional future channel for “screen changed” notifications; polling `GET /screen` is enough for v1.
4. **Global variable introspection** — out of scope; too fragile and concurrency-heavy. Prefer observable UI state.
5. **Official MCP server** — separate repo or `tools/fpas-tui-mcp`; document as consumer of this HTTP API only.

## Success criteria (when implemented)

1. `fpas run --debug-port 9333 examples/pascal/tui/minimal_application.fpas` serves `/health`.
2. `POST /events/key` with `Escape` causes the app to quit when a handler calls `HostRequestQuit`.
3. `GET /screen` returns the same compact lines as the test runner's `screen_snapshot()` after paint.
4. `GET /views` returns registered host views for an example using `HostRegisterView` or menu bar widgets.
5. No regression in `fpas test` when debug port is disabled.
6. Document link from [`docs/rust/tui-terminal-checklist.md`](../../rust/tui-terminal-checklist.md) as optional automation aid.
