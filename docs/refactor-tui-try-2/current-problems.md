# Current problems (try 1 — historical)

Analysis of the **pre-rewrite** `Std.Tui` facade. The try-2 branch removed reconcile, snapshot state, and the root bridge; this document remains as baseline context only. For open items see [remaining-work.md](remaining-work.md).

## Architecture: dual state

```text
Pascal Application.Create*
    → TurboVisionState.objects: HashMap<u32, TurboVisionObject>   ← authoritative for structure
    → pending_reconcile flag
    → turbo_vision_rebuild_desktop() wipes desktop and repopulates from snapshot
    → live turbo_vision::Application on Worker.live_turbo_vision_app
```

**Symptom:** Every structural change (`AddChild`, `AddWindow`, `SetMenus`, …) marks the tree dirty. During `Run`, `turbo_vision_reconcile_after_step` may rebuild the entire desktop ([`reconcile.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/reconcile.rs)).

**Cost:** Two representations of the same UI must stay in sync. Bugs show up as stale focus, wrong z-order, or missing children after live mutations.

## Bridge size and shape

| Metric | Value |
| --- | --- |
| Rust modules in `execute/io/tui/` | 41 |
| Approximate LOC | ~6,500 |
| `Bridged*` adapter views | 3 (`bridged_check_box`, `bridged_outline`, `bridged_radio_button`) |
| Headless-specific paths | `headless_tv_draw.rs`, `HeadlessTvApp`, duplicate chrome sync |

The bridge grew to paper over the snapshot/live split (`live_patch.rs`, `live_view_ids`, `live_child_root_view_ids`) instead of calling upstream directly.

## API ergonomics

Current Pascal code requires repeating `App` and using a flat `Application.*` namespace:

```pascal
var DialogHandle: Dialog := Application.CreateDialog(App, Bounds(...), 'Title');
var ButtonHandle: Button := Application.CreateButton(App, Bounds(...), 'OK', Command.Quit);
Application.AddChild(App, DialogHandle, ButtonHandle);
Application.OnCommand(App, OnCommand);
Application.Run(App);
```

Upstream equivalent is shorter and groups by view type:

```rust
let mut dialog = Dialog::new_modal(bounds, "Title");
dialog.add(Box::new(Button::new(bounds, "OK", CM_QUIT, false)));
app.exec_view(Box::new(dialog));
```

The FPAS API neither matches upstream naming nor reduces ceremony.

## Command ID translation

[`command_map.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/command_map.rs) maintains:

- A list of ~50 reserved upstream `CM_*` ids
- An offset band `0x8000` for colliding application commands
- Round-trip translation in `callbacks.rs`

**Symptom:** IDE About uses `100` (`CM_ABOUT`) with special offset behavior documented in [types.md](../pascal/std/tui/app/types.md). Application authors must understand collision rules.

**Try-2 approach:** Pascal uses the same `CM_*` values as upstream. Application-specific commands use a documented private range (e.g. `4096..`) or upstream’s conventional user band — no runtime offset.

## Headless testing complexity

Try-1 maintains:

- A separate `HeadlessTvApp` built from the FPAS snapshot ([`headless_tv_draw.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/headless_tv_draw.rs))
- `Pump` + command queue ([`commands.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/commands.rs))
- Test stubs: `TestSetDialogResult`, `TestSetFileDialogResult`, `TestClickButton`, `TestClickMouse`, `TestDispatchMenuCommand`

Many tests exist only to validate the bridge ([`tests/tui/controls/`](../tests/tui/controls/)) — 37 files, several for `SetText`, reconcile, and reserved-command behavior.

## Documentation drift risk

The skill [`.agents/skills/turbo-vision-4-rust/SKILL.md`](../../.agents/skills/turbo-vision-4-rust/SKILL.md) currently says “do not mirror Rust one-to-one,” while contributors still need a large [vm-bridge.md](../pascal/std/tui/app/vm-bridge.md) module table (40+ intrinsics). The docs describe bridge mechanics users should not need to know.

## What is worth keeping from try-1

| Piece | Keep? | Notes |
| --- | --- | --- |
| One live session per `Open…Close` | Yes | [`session_app.rs`](../../crates/fpas-vm/src/vm/execute/io/tui/session_app.rs) pattern |
| Shared session for `Run` / modals | Yes | Avoid `Application::new()` per dialog |
| `msgbox.rs` / upstream `message_box` | Yes | Already thin |
| Menu/status record types | Yes | FPAS-friendly data; build upstream in Rust |
| `OpenForTest` + screen asserts via `Std.Test` | Yes | Adapt to new headless path |
| `TurboVisionObject` snapshot | **No** | Delete |
| `reconcile` / `live_patch` / `Bridged*` | **No** | Delete |
| `command_map` offset band | **No** | Delete |
| `Application.Create*` namespace | **No** | Replace with view record APIs |

## Root cause summary

The try-1 design treats Pascal as the owner of a **retained scene description** and Turbo Vision as a **projection target**. That made sense when exploring headless testing, but it fights upstream’s ownership model. Try-2 inverts this: **Turbo Vision owns the tree; Pascal holds ids and calls mutating operations.**
