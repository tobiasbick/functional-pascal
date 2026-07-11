# Std.Tui VM bridge

Contributor map for the Pascal-to-VM bridge. User-facing API docs live in [README.md](README.md) and sibling pages under `app/`.

**Upstream:** [`turbo-vision`](https://crates.io/crates/turbo-vision) 2.0 from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) (git tag `v2.0.0` until crates.io publishes 2.x).

## Architecture

```text
Pascal Application.* / View.* / Desktop.Add
    → VM intrinsics (crates/fpas-vm/src/vm/execute/io/tui/)
    → Try2Session + ViewRegistry (try2/session.rs, try2/registry.rs)
    → turbo_vision::app::Application on Worker.live_turbo_vision_app
         one live instance per Open … Close on the main worker
```

| Concern | Location |
| --- | --- |
| Session lifecycle | `try2/lifecycle.rs`, `try2/application_intrinsics.rs`, `try2/session_app.rs` |
| Try-2 view registry | `try2/session.rs`, `try2/registry.rs`, `try2/views/` |
| Run loop | `try2/run.rs`, `try2/input_events.rs` |
| Command callbacks | `try2/events.rs` — upstream `CM_*` ids pass through unchanged |
| Chrome | `try2/chrome.rs`, `try2/chrome_input.rs`, `try2/chrome_layout.rs` |
| Modals | `try2/modals.rs`, `try2/message_box.rs`, `try2/file_dialog.rs` |
| Headless paint | `try2/headless_draw.rs` |
| Headless tests | `try2/testing.rs` |
| Bridged live views | `try2/bridged_check_box.rs`, `try2/bridged_outline.rs`, `try2/bridged_radio_button.rs` — upstream control types without a downcast hook |
| Handle records | `try2/handles.rs`, `try2/handle_records.rs` |
| Pascal `CM_*` constants | `crates/fpas-std/src/tui/cm_constants.rs` |

## Turbo Vision bump checklist

On every `turbo-vision` tag or revision bump in `Cargo.lock`:

1. Compare `fpas-std/src/tui/cm_constants.rs` with upstream `core::command` and update the exported subset as needed.
2. Run `fpas test tests/tui/` and `fpas test apps/ide/tests/`.

After any bridge change, also run [terminal checklist](../terminal-checklist.md).

## Representative intrinsics

| Pascal symbol | Intrinsic family |
| --- | --- |
| `Application.Open` / `New` | `ApplicationOpen` |
| `Application.Close` | `ApplicationClose` |
| `Application.Run` | `ApplicationRun` / `ApplicationRunWithOnCommand` |
| `Application.ExecView` | `ApplicationExecView` |
| `Dialog.NewModal` | `DialogNewModal` |
| `Button.New` | `ButtonNew` |
| `Dialog.Add` / `Window.Add` | polymorphic builtins → attach intrinsics |
| `Desktop.Add` | `DesktopAdd` |
| `Application.MessageBox` | `MessageBox` |
| `Application.RunFileDialog` | `RunFileDialog` |
| `Application.SetMenuBar` / `SetStatusLine` | `SetMenuBar` / `SetStatusLine` |
| `Application.OnKey` / `OnMouse` | `RegisterOnKey` / `RegisterOnMouse` |
| `Application.TestInjectCommand` | `Try2InjectCommand` (interim; prefer `Test.InjectCommand`) |
| `Test.InjectCommand` | `Try2InjectCommand` (preferred Pascal name) |
| `Application.TestInjectKeyboard` | `Try2InjectKeyboard` (interim; prefer `Test.InjectKeyboard`) |
| `Test.InjectKeyboard` | `Try2InjectKeyboard` (preferred Pascal name) |
| `Application.TestClickButton` | `TestClickButton` |
| `Test.Click` | `TestClickButton` (preferred Pascal name) |
| `Application.TestDispatchMenuCommand` | `TestDispatchMenuCommand` |
| `Test.DispatchMenu` | `TestDispatchMenuCommand` (preferred Pascal name) |

Interim test helpers are scheduled for rename during Phase 7 closure — see [remaining-work.md](../../../../refactor-tui-try-2/remaining-work.md) stream B.

Full opcode tables: `crates/fpas-bytecode/src/intrinsic/tui/variants/try2.inc` and `widgets.inc`.

## See also

- [Application](README.md)
- [Native testing](testing.md)
- [Refactor status](../../../../refactor-tui-try-2/status.md)
