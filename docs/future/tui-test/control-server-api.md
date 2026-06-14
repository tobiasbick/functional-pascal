# TUI control server — HTTP API (proposal)

**Status:** design only — not implemented.

Normative API draft for the localhost debug server described in [`README.md`](README.md). Operations are **independent**: clients may call read-only endpoints without ever injecting events, and may inject events without reading state.

## Transport

| Property | Value |
| -------- | ----- |
| Protocol | HTTP/1.1 |
| Bind address | `127.0.0.1` only |
| Default port | `9333` (configurable) |
| Request body | JSON (`Content-Type: application/json`) |
| Response body | JSON (`Content-Type: application/json; charset=utf-8`) |
| Errors | JSON object with `error` string and optional `hint` |

Activation (proposed):

```sh
fpas run --debug-port 9333 path/to/app.fpasprj
# or
FPAS_DEBUG_PORT=9333 fpas path/to/program.fpas
```

The server starts when the VM begins execution and stops when the program exits or the debug port is disabled.

## Conventions

### Coordinates

- **Screen / view rects:** zero-based terminal cell coordinates (`x`, `y`, `width`, `height`), matching the host [`ViewRect`](../../../crates/fpas-std/src/tui/view/mod.rs).
- **Console mouse events** in write requests use **one-based** columns and rows, matching `Std.Console.Event` (`mouseColumn`, `mouseRow`).

### View identifiers

- `view_id` is the opaque integer returned by `Application.HostRegisterView` and related host APIs.
- `-1` means “no focused view” (same as `Application.HostQueryFocusedViewId`).

### Event kind strings

Key and mouse enumerations reuse the same string labels as [`*.script.toml`](../../../crates/fpas-cli/src/test_script/parse.rs) sidecars (for example `Escape`, `Tab`, `Down`, `Left`).

---

## Read-only endpoints

These endpoints never mutate VM or TUI state.

### `GET /health`

Liveness and high-level runtime flags.

**Response 200**

```json
{
  "status": "running",
  "surface": "tui",
  "tui_active": true,
  "headless": false,
  "uptime_ms": 1240
}
```

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `status` | string | `"running"`, `"idle"`, or `"exited"` |
| `surface` | string | `"tui"`, `"graph"`, or `"console"` — which hosted loop owns input |
| `tui_active` | boolean | `Application.Run` (or equivalent host loop) is active |
| `headless` | boolean | No real terminal writer attached |
| `uptime_ms` | integer | Milliseconds since server start |

---

### `GET /screen`

Logical CRT screen snapshot (characters only). Same source as `vm.screen_snapshot()` and `*.expect.screen` golden files.

**Query parameters**

| Name | Default | Description |
| ---- | ------- | ----------- |
| `compact` | `true` | When true, trim trailing spaces and leading/trailing blank rows (same as `ScreenSnapshot::compact_lines`) |
| `raw` | `false` | When true, return full fixed-size row grid including empty rows |

**Response 200 (`compact=true`)**

```json
{
  "width": 80,
  "height": 25,
  "lines": [
    "Press Escape to quit"
  ]
}
```

**Response 200 (`raw=true`)**

```json
{
  "width": 80,
  "height": 25,
  "rows": [
    "Press Escape to quit                                                            ",
    "                                                                                "
  ]
}
```

---

### `GET /screen/cells`

Rectangular region of the CRT back buffer with per-cell character and color indices.

**Query parameters**

| Name | Required | Description |
| ---- | -------- | ----------- |
| `x` | yes | Left column (zero-based) |
| `y` | yes | Top row (zero-based) |
| `w` | yes | Width in columns |
| `h` | yes | Height in rows |

**Response 200**

```json
{
  "x": 0,
  "y": 0,
  "width": 3,
  "height": 1,
  "cells": [
    [
      { "ch": "H", "fg": 7, "bg": 0 }
    ]
  ]
}
```

`cells[row][col]` matches host test helpers on `ConsoleState`. Values outside the screen clip to the visible area; empty clip returns zero width/height.

---

### `GET /focus`

Focused view and modal stack summary.

**Response 200**

```json
{
  "focused_view_id": 3,
  "modal_depth": 1
}
```

Maps to `Application.HostQueryFocusedViewId` and `Application.HostModalDepth`.

---

### `GET /views`

Host-managed view tree snapshot.

**Response 200**

```json
{
  "roots": [1, 4],
  "views": [
    {
      "id": 1,
      "parent_id": null,
      "rect": { "x": 0, "y": 0, "width": 80, "height": 1 },
      "children": [],
      "in_focus_chain": true,
      "widget": "menu_bar"
    },
    {
      "id": 3,
      "parent_id": 2,
      "rect": { "x": 5, "y": 5, "width": 20, "height": 10 },
      "children": [],
      "in_focus_chain": true,
      "widget": null
    }
  ]
}
```

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `roots` | array of integer | Root view ids in paint / z-order |
| `views[].rect` | object | Absolute terminal rectangle |
| `views[].in_focus_chain` | boolean | View participates in Tab / Shift+Tab traversal |
| `views[].widget` | string or null | Host widget kind when known (`menu_bar`, `status_bar`, `solid_fill`, …) |

Views with Pascal-only `OnViewPaint` handlers and no host widget use `"widget": null`.

---

### `GET /views/{id}`

Single view lookup. Returns **404** when the id is unknown.

**Response 200**

```json
{
  "id": 3,
  "parent_id": 2,
  "rect": { "x": 5, "y": 5, "width": 20, "height": 10 },
  "children": [],
  "in_focus_chain": true,
  "widget": null,
  "focused": true
}
```

---

### `GET /stdout`

Captured `WriteLn` / line-buffered stdout lines (test runner capture), in emission order.

**Response 200**

```json
{
  "lines": [
    "Modal depth: 1"
  ]
}
```

---

## Write endpoints

These endpoints enqueue input or request host actions. They return immediately after the event is queued; they do **not** wait for FPAS handlers to run unless documented otherwise.

### `POST /events/key`

Queue a structured key event for the hosted TUI / console event pump.

**Request body**

```json
{
  "kind": "Escape",
  "ch": null,
  "shift": false,
  "ctrl": false,
  "alt": false,
  "meta": false
}
```

| Field | Type | Required | Description |
| ----- | ---- | -------- | ------------- |
| `kind` | string | yes | Key kind name (same as script sidecar) |
| `ch` | string or null | no | Single character when kind is `Char` |
| `shift`, `ctrl`, `alt`, `meta` | boolean | no | Modifier flags (default false) |

**Response 202**

```json
{
  "queued": true,
  "queue_depth": 2
}
```

Implementation maps to `Vm::push_console_event(ConsoleEvent::key(...))`.

---

### `POST /events/mouse`

Queue a mouse event.

**Request body**

```json
{
  "action": "Down",
  "button": "Left",
  "x": 10,
  "y": 5,
  "shift": false,
  "ctrl": false,
  "alt": false,
  "meta": false
}
```

`x` and `y` are **one-based**, matching `Std.Console.Event`.

**Response 202** — same shape as `/events/key`.

---

### `POST /events/resize`

Queue a terminal resize event.

**Request body**

```json
{
  "width": 120,
  "height": 40
}
```

**Response 202** — same shape as `/events/key`.

---

### `POST /events/paste`

Queue a paste event.

**Request body**

```json
{
  "text": "hello"
}
```

**Response 202** — same shape as `/events/key`.

---

### `POST /events/focus`

Queue terminal focus gained or lost.

**Request body**

```json
{
  "gained": true
}
```

**Response 202** — same shape as `/events/key`.

---

### `POST /input/readln`

Queue one line for the next blocking `Std.Console.ReadLn` (or line-buffered read). Same as script event `type = "readln"`.

**Request body**

```json
{
  "line": "user input"
}
```

**Response 202**

```json
{
  "queued": true
}
```

---

### `POST /input/readkey`

Queue raw characters for CRT-style `ReadKey` / `Read` tests.

**Request body**

```json
{
  "chars": "abc"
}
```

**Response 202** — same shape as `/input/readln`.

---

### `POST /control/quit`

Request cooperative shutdown via the TUI host (`Application.HostRequestQuit` semantics).

**Request body**

```json
{}
```

Optional field:

| Field | Type | Description |
| ----- | ---- | ----------- |
| `reason` | string | Diagnostic label for logs only; not passed to FPAS |

**Response 202**

```json
{
  "requested": true
}
```

---

## Optional control endpoints (v2)

Not required for the first useful release. Listed here so read/write endpoints stay minimal in v1.

### `POST /control/step`

Process exactly one hosted event cycle (`TuiHostProcessNext` equivalent), then return updated snapshots.

**Request body**

```json
{
  "timeout_ms": 0
}
```

**Response 200**

```json
{
  "processed": true,
  "dispatch_tag": 16,
  "focus": { "focused_view_id": 3, "modal_depth": 0 },
  "screen": {
    "width": 80,
    "height": 25,
    "lines": ["..."]
  }
}
```

Enables deterministic “inject → step → assert” loops without racing the real terminal.

### `POST /control/redraw`

Force a hosted redraw (`TuiHostDispatchRedraw`) without injecting input.

**Response 202**

```json
{
  "scheduled": true
}
```

---

## Error responses

**404** — unknown view id or path.

```json
{
  "error": "view not found",
  "hint": "Call GET /views for the current registry."
}
```

**409** — server not ready (program exited, TUI not active, wrong surface).

```json
{
  "error": "tui not active",
  "hint": "Start a program that calls Application.Run with --debug-port enabled."
}
```

**400** — invalid JSON or unknown enum string.

```json
{
  "error": "unknown key kind 'Esc'",
  "hint": "Use Escape, Tab, Char, … See script sidecar documentation."
}
```

---

## Example sessions

### Read-only inspection

```sh
curl -s http://127.0.0.1:9333/health
curl -s http://127.0.0.1:9333/screen
curl -s http://127.0.0.1:9333/views
curl -s http://127.0.0.1:9333/focus
```

### Inject Escape and read screen

```sh
curl -s -X POST http://127.0.0.1:9333/events/key \
  -H 'Content-Type: application/json' \
  -d '{"kind":"Escape"}'
curl -s http://127.0.0.1:9333/screen
```

### Playwright-style script (pseudo-code)

```typescript
async function dismissModal(port: number) {
  await fetch(`http://127.0.0.1:${port}/events/key`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ kind: "Escape" }),
  });
  const screen = await fetch(`http://127.0.0.1:${port}/screen`).then(r => r.json());
  assert(screen.lines.includes("Goodbye"));
}
```

---

## MCP wrapper (future)

A Cursor MCP server can expose thin tools that call this HTTP API (`fpas_tui_screen`, `fpas_tui_key`, `fpas_tui_views`). The MCP layer should not define a second protocol; it forwards to the endpoints above.

---

## Mapping to existing implementation (when built)

| API | Existing hook |
| --- | ------------- |
| Event POST endpoints | [`Vm::push_console_event`](../../../crates/fpas-vm/src/vm/mod.rs), [`test_script/console.rs`](../../../crates/fpas-cli/src/test_script/console.rs) |
| Readln / readkey POST | [`Vm::push_readln_input`](../../../crates/fpas-vm/src/vm/mod.rs), script `Readln` / `ReadkeyChars` |
| `/screen` | [`Vm::screen_snapshot`](../../../crates/fpas-vm/src/vm/mod.rs), [`ScreenSnapshot`](../../../crates/fpas-std/src/console/snapshot.rs) |
| `/focus` | `TuiHostQueryFocusedViewId`, `TuiHostModalDepth` intrinsics |
| `/views` | [`ViewRegistry`](../../../crates/fpas-std/src/tui/view/mod.rs) in `TuiState` |
| `/control/quit` | `TuiHostRequestQuit` intrinsic path |
