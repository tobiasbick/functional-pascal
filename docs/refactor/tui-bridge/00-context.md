# TUI bridge — current architecture (reference)

- [x] Update this page when a refactor item materially changes the bridge (keep in sync)

Living summary of how FPAS integrates [turbo-vision 2.0](https://github.com/aovestdipaperino/turbo-vision-4-rust). Read this before starting any item in this folder.

## Goal

Expose a **Pascal-native** `Std.Tui` API (`Application.Create*`, `OnCommand`, `Run`) over upstream Turbo Vision. Do **not** expose Rust `View` traits, builders, or ownership to FPAS programs.

## Data flow

```text
Pascal Application.*
    → VM intrinsics (crates/fpas-vm/src/vm/execute/io/tui/mod.rs)
    → TurboVisionState (crates/fpas-vm/src/vm/shared/tui.rs)
         HashMap<u32, TurboVisionObject>  — authoritative handle graph
    → projection at Run / reconcile
    → turbo_vision::app::Application  — one live instance per Open…Close on main worker
         (Worker.live_turbo_vision_app in session_app.rs)
```

## What delegates to turbo-vision (interactive)

| Concern | Upstream | FPAS bridge entry |
| --- | --- | --- |
| Widget types | `views::*` | Built in `tv_views.rs`, `menu_build.rs` from FPAS snapshots |
| Event loop | `get_event`, `handle_event`, desktop cleanup | `session_app.rs` (`turbo_vision_drive_live_interactive_loop`) |
| Menu / status | `MenuBar`, `StatusLine` | `navigation.rs`, `chrome_layout.rs` |
| Modal execute | `Dialog::execute` | `exec_dialog.rs` (on live session) |
| Message box | `helpers::msgbox::message_box` | `msgbox.rs` + `Application.MessageBox` — [done/03-about-message-box.md](done/03-about-message-box.md) |
| File picker | `FileDialog` | `file_dialog.rs` (on live session) |
| Command ids | Borland `CM_*` in `core/command.rs` | `command_map.rs` + `fpas-std` `command_ids.rs` |

## What FPAS reimplements today

| Concern | Location | Notes |
| --- | --- | --- |
| Retained handles | `shared/tui.rs`, `control_create.rs` | `Create*` only writes FPAS records; upstream widgets rebuilt later |
| Headless paint | `headless_tv_draw.rs` | Upstream TV `draw` → CRT export ([done/04-headless-test-util.md](done/04-headless-test-util.md)) |
| Headless commands | `commands.rs`, `tv_run.rs` | Queue + `Pump` instead of TV event loop |
| Full desktop rebuild | `reconcile.rs`, `tv_run.rs` | `pending_reconcile` → wipe desktop → repopulate |
| State cells | `turbo_vision_*_cell.rs`, `bridged_*.rs` | Sync checkbox/radio/list/input back to FPAS handles |
| Command offset band | `command_map.rs` | App-defined ids that collide with `CM_*` use `0x8000` band; `Command.*` pass through |

## Known duplication (refactor targets)

1. **Headless input** — queue + `test_mouse.rs` hit-test vs TV `push_event` → follow-up in [done/04-headless-test-util.md](done/04-headless-test-util.md)

**Done:** single live `Application` per FPAS session — [done/02-single-tv-session.md](done/02-single-tv-session.md). IDE About via upstream `message_box` — [done/03-about-message-box.md](done/03-about-message-box.md).

## Bridge size (approximate)

| Area | Files | LOC |
| --- | --- | --- |
| `crates/fpas-vm/src/vm/execute/io/tui/` | 33 | ~4 900 |
| Related cells + `shared/tui.rs` + `tui_run.rs` + `worker.rs` | ~5 | ~200 |
| **Total** | ~38 | ~5 100 |

## Public Pascal surface

Documented under [docs/pascal/std/tui/](../../pascal/std/tui/README.md). Internal/host APIs in `fpas-std` session code are **not** part of the Turbo Vision facade.

## Upstream dependency

Workspace pin (until crates.io publishes 2.x):

```toml
turbo-vision = { git = "https://github.com/aovestdipaperino/turbo-vision-4-rust", tag = "v2.0.0" }
```

## Verification baseline

After any bridge change:

```text
cargo fmt
cargo build
cargo test --workspace
cargo run -q -p fpas-cli -- test tests/tui/controls/
cargo run -q -p fpas-cli -- test apps/ide/tests/
cargo run -q -p fpas-cli -- test apps/ide/tests/
```
