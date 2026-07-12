# Std.Tui VM bridge

Contributor map for the Pascal-to-VM bridge. User-facing API docs live in [README.md](README.md) and sibling pages under `app/`.

**Upstream:** [`turbo-vision`](https://crates.io/crates/turbo-vision) 2.0 from [turbo-vision-4-rust](https://github.com/aovestdipaperino/turbo-vision-4-rust) (git tag `v2.0.0` until crates.io publishes 2.x).

## Architecture

```text
Pascal Application.* / View.* / Desktop.Add
    → VM intrinsics (crates/fpas-vm/src/vm/execute/io/tui/)
    → TurboVisionSession + ViewRegistry (bridge/session.rs, bridge/registry.rs)
    → turbo_vision::app::Application on Worker.live_turbo_vision_app
         one live instance per Open … Close on the main worker
```

| Concern | Location |
| --- | --- |
| Session lifecycle | `bridge/lifecycle.rs`, `bridge/application_intrinsics.rs`, `bridge/session_app.rs` |
| Turbo Vision bridge view registry | `bridge/session.rs`, `bridge/registry.rs`, `bridge/views/` |
| Run loop | `bridge/run.rs`, `bridge/input_events.rs` |
| Command callbacks | `bridge/events.rs` — upstream `CM_*` ids pass through unchanged |
| Chrome | `bridge/chrome.rs`, `bridge/chrome_input.rs`, `bridge/chrome_layout.rs` |
| Modals | `bridge/modals.rs`, `bridge/message_box.rs`, `bridge/file_dialog.rs` |
| Headless paint | `bridge/headless_draw.rs` |
| Headless tests | `bridge/testing.rs` |
| Bridged live views | `bridge/bridged_check_box.rs`, `bridge/bridged_outline.rs`, `bridge/bridged_radio_button.rs` — upstream control types without a downcast hook |
| Handle records | `bridge/handles.rs`, `bridge/handle_records.rs` |
| Pascal `CM_*` constants | `crates/fpas-std/src/tui/cm_constants.rs` |

## Turbo Vision bump checklist

On every `turbo-vision` tag or revision bump in `Cargo.lock`:

1. `fpas-std/build.rs` regenerates the selected `CM_*` values directly from upstream `core::command`. Update its explicit list only when the Pascal command-constant surface should grow or shrink.
2. Run `fpas test tests/tui/` and `fpas test apps/ide/tests/`.

After any bridge change, also run [terminal checklist](../terminal-checklist.md).

## Representative intrinsics

| Pascal symbol | Intrinsic family |
| --- | --- |
| `Application.Open` / `New` | `ApplicationOpen` |
| `Application.Close` | `ApplicationClose` |
| `Application.Run` | `ApplicationRun` / `ApplicationRunWithOnCommand` |
| `Application.Configure` | `ApplicationConfigure` |
| `Application.ExecView` | `ApplicationExecView` |
| `Dialog.NewModal` | `DialogNewModal` |
| `Button.New` | `ButtonNew` |
| `Dialog.Add` / `Window.Add` | polymorphic builtins → attach intrinsics |
| `Desktop.Add` | `DesktopAdd` |
| `EditorWindow.New` | `EditorWindowNew` |
| `Application.MessageBox` | `MessageBox` |
| `Application.RunFileDialog` | `RunFileDialog` |
| `Application.SetMenuBar` / `SetStatusLine` | `SetMenuBar` / `SetStatusLine` |
| `Application.OnKey` / `OnMouse` | `RegisterOnKey` / `RegisterOnMouse` |
| `Test.InjectCommand` | `TestInjectCommand` |
| `Test.InjectKeyboard` | `TestInjectKeyboard` |
| `Test.Click` | `TestClickButton` |
| `Test.DispatchMenu` | `TestDispatchMenuCommand` |

Headless stub/coordinate helpers remain on `Application` (`TestClickMouse`, `TestSetDialogResult`, `TestSetFileDialogResult`). Three checkbox/radio/outline bridge adapters remain until upstream read-back — see [tui-bridged-readback.md](../../../../future/tui-bridged-readback.md). Agents: periodic upstream check in [AGENTS.md](../../../../../AGENTS.md#upstream-watch--turbo-vision-4-rust-read-back-stream-a); contributors: [AI_CONTRIBUTING.md](../../../../../AI_CONTRIBUTING.md#good-entry-points).

Full opcode tables: `crates/fpas-bytecode/src/intrinsic/tui/variants/bridge.inc` and `widgets.inc`.

## See also

- [Application](README.md)
- [Native testing](testing.md)
- [Remaining upstream adapters](../../../../future/tui-bridged-readback.md)
