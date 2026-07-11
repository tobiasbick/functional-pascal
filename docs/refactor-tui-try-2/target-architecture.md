# Target architecture

## Data flow

```text
┌─────────────────────────────────────────────────────────────────┐
│ Pascal program                                                   │
│   Dialog.New / Dialog.Add / Application.ExecView / Application.Run │
└────────────────────────────┬────────────────────────────────────┘
                             │ VM intrinsics (thin)
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ Worker                                                           │
│   live_turbo_vision_app: Option<turbo_vision::app::Application>  │
│   view_registry: HashMap<FpasViewId, TvViewRef>                │
│   tui_callbacks: OnCommand / OnKey / OnMouse                     │
└────────────────────────────┬────────────────────────────────────┘
                             │ direct calls
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ turbo_vision 2.0                                                 │
│   Application { desktop, menu_bar, status_line, terminal, … }   │
│   Group::add(Box<dyn View>)                                      │
│   Application::exec_view / get_event / run                       │
└─────────────────────────────────────────────────────────────────┘
```

No arrow back from Turbo Vision into a parallel FPAS widget enum for structure.

## View identity

### FPAS side

Each widget record carries an opaque integer handle assigned by the VM:

```pascal
type
  View = record
    id: integer;
  end;

  Dialog = View;   { same representation; sema distinguishes types }
  Button = View;
  …
```

Sema still types `Dialog` and `Button` separately to prevent attaching a button to a menu bar, but the runtime representation is a `FpasViewId: u32`.

### Rust side

```rust
struct TvViewRef {
    view_id: turbo_vision::views::ViewId,
    kind: ViewKind,  // Dialog, Button, … — for diagnostics and read-back
    // optional: weak parent link for validation only
}

struct ViewRegistry {
    next_id: u32,
    entries: HashMap<u32, TvViewRef>,
}
```

`ViewRegistry` is **indexing only**. It does not duplicate bounds, text, or children — those live on upstream views. Read-back (`InputLine.Text`, `ListBox.Selection`, …) uses `view_id` to find the live view via `desktop` traversal or stored `ViewId` on the parent group.

## Session lifecycle

Matches current lifecycle semantics (see [session.md](../pascal/std/tui/session.md)) but simplified:

| Phase | Behavior |
| --- | --- |
| `Application.New` / `Open` | Allocate `TuiSession` + empty `ViewRegistry`. **Do not** init terminal. |
| `OpenForTest(w, h)` | Allocate session + headless `Terminal` via upstream test backend. |
| First interactive call (`Run`, `ExecView`, `RunFileDialog`, `MessageBox`) | `Application::new()` or headless equivalent; store in `live_turbo_vision_app`. |
| `Run` | `app.run()` loop (or thin wrapper calling `get_event` + FPAS callbacks). |
| `Quit` | Set `app.running = false`. |
| `Close` | Drop live app, clear registry, release terminal. |

One upstream `Application` per FPAS `Open…Close` on the main worker — retain from try-1.

## Mutations

| Operation | Implementation |
| --- | --- |
| `Dialog.New` | Construct upstream `Dialog`, `registry.insert`, return handle. **Do not** add to desktop until `ExecView` or explicit `Desktop.Add`. |
| `Dialog.Add(Child)` | Resolve parent `ViewId` → `&mut dyn Group` → `add(Box::new(child))`. |
| `Button.SetText` | Resolve view → `as_any_mut` downcast or stored cell on wrapper **only if upstream lacks setter** (prefer upstream setters). |
| `Application.SetMenuBar` | `app.set_menu_bar(menu_bar)`; drop previous bar from registry if needed. |
| `Desktop.Add(Window)` | `app.desktop.add(Box::new(window))`. |

Structural changes apply **immediately** on the live tree. No `pending_reconcile`.

## Modal execution

```text
Application.ExecView(App, Dialog)
  → turbo_vision_with_live_app
  → app.exec_view(boxed_dialog)   // upstream owns modal loop
  → return CommandId (end_state)
```

Read-back after modal (`InputLine.Text`, `CheckBox.Checked`, …) queries the live view still registered under the handle. Handles remain valid until `Close` or explicit destroy (if we add `View.Free` later).

## Event dispatch

```text
app.get_event / run inner loop
  → event.what == Command
  → try2_command_for_callback (identity — no command_map offset)
  → dispatch OnCommand(App, command_id)
  → if CM_QUIT or Application.Quit → stop run loop
```

**Coexistence:** `Application.Run` uses try-2 loop when `Try2Session` is open and try-1 `turbo_vision.objects` is empty; otherwise try-1 run + offset translation still apply.

Unhandled keyboard/mouse after desktop dispatch → `OnKey` / `OnMouse` (live try-2 run delegates unhandled input to try-1 helpers today).

## Headless architecture

**Goal:** one code path for widget construction; headless differs only in terminal backend.

```text
OpenForTest(w, h)
  → FPAS console + TvHeadlessBackend (current: HeadlessTvApp in headless_tv_draw.rs)
  → same Group::add / exec_view / run_step code paths as interactive mode
  → target: single headless Application::new() terminal (phase 7 consolidation)
```

### Test input

| Mechanism | Use |
| --- | --- |
| `app.put_event(Event::…)` | Inject keys, commands, mouse |
| Upstream `test-util` helpers | Prefer when they exist for a scenario |
| `Test.Click(Button)` | Thin wrapper: resolve bounds → synthesize mouse event → `handle_event` |
| `Test.Click` | Headless button click (`try2/testing.rs`) |
| `Test.InjectCommand` | Headless command injection for run smoke tests |
| `Test.SetModalResult(CommandId)` | **Remove** when all modals use injected events (modal smoke already does) |

Target: delete `TestSetDialogResult` / `TestSetFileDialogResult` stubs by driving real modal loops in headless mode.

### Screen assertions

Unchanged: [`Std.Test`](../../docs/pascal/std/testing/test.md) `AssertScreenLine` / `AssertScreenCell` on the console back buffer after draw.

## Worker fields (target)

```rust
// crates/fpas-vm/src/vm/worker.rs (current + target)
pub(crate) live_turbo_vision_app: Option<TurboVisionApplication>,
pub(crate) try2: Try2Session,              // current coexistence bridge
// target after phase 7: tv_view_registry on Worker or folded into session
// TuiState retains: session, on_command, on_key, on_mouse, quit_requested
// Remove after phase 7: turbo_vision: TurboVisionState with TurboVisionObject map
```

## Concurrency

Same as try-1: live app is `!Send`, main worker only. Document in lifecycle page.

## Error handling

| Failure | Diagnostic |
| --- | --- |
| Use handle after `Close` | Runtime error: invalid view handle |
| `Add` to wrong parent type | Runtime error: expected `Dialog` or `Window` |
| Terminal init failure | Runtime error with hint to use `OpenForTest` |
| Upstream `valid()` returns false | Propagate as blocked command (no FPAS callback) |

## Z-order and desktop children

Use upstream desktop order. `Desktop.Add` append order matches try-1’s “newer windows above older dialogs” policy if we add windows in the same sequence. Document that z-order equals add order.

## Palette / chrome

Keep chrome layout logic in one module (`chrome.rs`): menu bar and status line bounds follow upstream `set_menu_bar` / `set_status_line` — delete duplicate layout in FPAS snapshot builders.

## What we do not build

- FPAS-side scene graph queries (`QuerySceneGraph`, retained clip, etc.)
- Incremental reconcile
- Custom `BridgedButton` unless a live `SetText` bug forces a minimal patch — try without wrappers first
