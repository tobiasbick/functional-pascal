# Target API

**Status: archival planning sketch** (pre-migration). For the implemented API, read
`docs/pascal/std/tui/`. This file records original intent and gaps vs what shipped.

## Implemented (summary)

| Area | Shipped as |
| --- | --- |
| Session | `Application.Open`, `Close`, `Run`, `Quit`, `OpenForTest` |
| Handles | `Window`, `Dialog`, `Button`, `StaticText`, `Memo`, `TextViewer`, `InputLine`, `ListBox`, `CheckBox`, `RadioButton`, `MenuBar`, `StatusLine` |
| Construction | `Application.Create*`, `AddChild`, `AddWindow`, `SetMenuBar`, `SetStatusLine` |
| Commands | `Command.Accept`, `Cancel`, `Close`, `Quit` + app-defined ids |
| Callbacks | `OnCommand`; optional `OnKey`, `OnMouse` (Turbo Vision path) |
| Modals | `ExecDialog` → `DialogResult`; `InputText`; `Checked`; `RunFileDialog` |
| Runtime mutation | `SetText`, `SetChecked`, `SetItems`, `SetTitle`, `SetMenus`, `SetStatusItems` |
| Menus | `Menu` / `MenuItem` records (multi-item, separators via `commandId = 0`) |
| Testing | `TestClickButton`, `TestDispatchMenuCommand`, `TestSetDialogResult`, screen queries |
| Hosted canvas (parallel) | `Application.Configure` + global handlers — **not** mixed with `Create*` |

Planning names that changed during implementation:

- `Command.Ok` → `Command.Accept`
- `Application.RunDialog` → `Application.ExecDialog`
- flat `MenuBarItem` → nested `Menu` / `MenuItem`
- spike `TuiDialog` / `TuiButton` → `Dialog` / `Button`

## Not implemented / deferred

| Planned idea | Status |
| --- | --- |
| `View` as a public umbrella handle | not shipped; use concrete widget handles |
| `Application.Remove` | not shipped |
| `Application.OnEvent` unified handler | not shipped; use `OnCommand` / `OnKey` / `OnMouse` |
| `Application.Size` | not shipped; use `QueryScreenSize` in tests |
| ListBox selection read-back | blocked on upstream `turbo-vision` 1.3.1 |
| RadioButton selection read-back after modal | planned — see [07-post-migration-improvements.md](07-post-migration-improvements.md) |
| Remove hosted `Configure` loop | deferred — `minimal_application.fpas` still uses it |

## Original shape (historical)

Host-owned handles for live UI objects; records for `Point`, `Size`, `Rect`; command integers for
actions. Naming rules: no `Host*` in public names; no Rust surface leaked to Pascal.

### Minimal spike (landed)

```pascal
uses Std.Tui;

procedure OnCommand(App: Application; CommandId: integer);
begin
  if CommandId = Command.Quit then
    Application.Quit(App)
end;

var App: Application := Application.Open();
var Win: Window := Application.CreateWindow(App, Bounds(5, 3, 50, 15), 'Demo');
var Btn: Button := Application.CreateButton(App, Bounds(18, 8, 30, 10), 'Quit', Command.Quit);
Application.AddChild(App, Win, Btn);
Application.AddWindow(App, Win);
Application.OnCommand(App, OnCommand);
Application.Run(App);
```

See `examples/pascal/tui/turbo_vision_window.fpas` and related examples for current spelling (`Bounds` record literals, dialog-centric demos).

## See also

- [Implementation phases](04-implementation-phases.md) — migration checklist
- [Post-migration improvements](07-post-migration-improvements.md) — remaining work
