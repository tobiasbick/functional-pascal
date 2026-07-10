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
| Session lifecycle | `lifecycle.rs`, `application.rs`, `session_app.rs` |
| Try-2 view registry | `try2/session.rs`, `try2/registry.rs`, `try2/views/` |
| Run loop | `try2/run.rs`, `tv_input_events.rs` |
| Command callbacks | `try2/events.rs` — upstream `CM_*` ids pass through unchanged |
| Chrome | `try2/chrome.rs`, `navigation.rs`, `chrome_layout.rs` |
| Modals | `try2/modals.rs`, `try2/message_box.rs`, `try2/file_dialog.rs`, `msgbox.rs`, `file_dialog.rs` |
| Headless paint | `headless_tv_draw.rs`, `try2/headless.rs` |
| Headless tests | `try2/testing.rs`, `testing.rs`, `test_mouse.rs` |
| Bridged live views | `bridged_*.rs` — checkbox/radio/input sync for try-2 attach path |
| Handle records | `handles.rs`, `handle_records.rs` |
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
| `Application.TestInjectCommand` | `Try2InjectCommand` (interim headless test) |
| `Application.TestInjectKeyboard` | `Try2InjectKeyboard` (interim headless test) |
| `Application.TestClickButton` | `TestClickButton` |

Full opcode tables: `crates/fpas-bytecode/src/intrinsic/tui/variants/try2.inc` and `widgets.inc`.

## See also

- [Application](README.md)
- [Native testing](testing.md)
- [Refactor status](../../../../refactor-tui-try-2/status.md)
