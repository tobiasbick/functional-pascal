# TUI live control and inspection (proposal)

**Status:** design only — not implemented.

Planning documents for an optional **localhost debug server** that exposes hosted `Std.Tui` state and lets external tools inject input events while a program runs. The goal is Playwright-style automation and interactive inspection without a real terminal protocol (PTY escape sequences) or a browser DevTools model.

## Motivation

Today's TUI tests rely on:

- **Pre-run script sidecars** (`<test>.script.toml`) that queue events before `vm.run()`.
- **Post-run golden files** (`*.expect.screen`) that compare the CRT back buffer.
- **In-program assertions** (`Std.Test`) and a few host query intrinsics (`Application.HostQueryFocusedViewId`, `Application.HostModalDepth`).

That works well for CI regression but not for:

- Inspecting state **while** a live app runs (for example the IDE).
- Stepping event-by-event between injections.
- External tools (curl, scripts, a future MCP wrapper) that only read or only write.

A debug server reuses the same VM input queues as scripted tests (`Vm::push_console_event`, `Vm::push_readln_input`) and the same screen snapshot path as the test runner (`vm.screen_snapshot()`), but exposes them over HTTP on loopback.

## Documents

| File | Contents |
| ---- | -------- |
| [`control-server-api.md`](control-server-api.md) | HTTP JSON API: read endpoints, write endpoints, schemas, examples |
| [`architecture.md`](architecture.md) | Host integration, threading, security, CLI activation, relation to sidecars |

## Non-goals (initial design)

- Not a `Std.Net` / `Std.Http` surface for FPAS programs.
- Not remote access (bind address is loopback only).
- Not a PTY or crossterm escape-sequence harness.
- Not a CSS-selector DOM; view handles and terminal rectangles are the query model.
- Not required for normal `fpas` / `fpas test` usage.

## Related documentation

| Document | Relevance |
| -------- | --------- |
| [`docs/pascal/std/tui-app.md`](../../pascal/std/tui-app.md) | Hosted TUI API, view handles, focus, modals |
| [`docs/pascal/std/test.md`](../../pascal/std/test.md) | `fpas test`, sidecars, golden screen |
| [`docs/rust/tui-terminal-checklist.md`](../../rust/tui-terminal-checklist.md) | Manual real-terminal verification |
| [`docs/future/tui-application-framework.md`](../tui-application-framework.md) | Phase 8 quality / scripted terminal work |

## Open decisions

See [`architecture.md`](architecture.md#open-decisions). Summary:

1. Default port and CLI flag naming (`--debug-port` vs `FPAS_DEBUG_PORT`).
2. Whether v1 includes `POST /control/step` or only fire-and-forget event injection.
3. Whether graph apps share the same server with a `/surface` discriminator or a separate port.
