# Verification

Definition of done for the try-2 TUI rewrite. **All items apply at phase 7 completion** — not the current coexistence branch. For branch progress see [status.md](status.md).

## Build and format

```bash
cargo fmt --check
cargo build --workspace
cargo clippy --workspace -- -D warnings   # optional but recommended
```

## Automated tests

```bash
cargo test --workspace
fpas fmt --check tests/tui/ apps/ide/ examples/pascal/tui/
fpas test tests/tui/
fpas test apps/ide/tests/
fpas test tests/                          # full regression suite
```

Or equivalent:

```bash
cargo test -p fpas-cli fpas_regression_suite_passes
```

## Grep invariants (no try-1 internals)

From repository root:

```bash
rg "TurboVisionObject|pending_reconcile|FPAS_TV_COMMAND_OFFSET|TuiCreateDialog|TuiAddChild" crates/
rg "Application\.CreateDialog|Application\.AddChild|Command\.Quit" docs/pascal/std/tui/
rg "bridged_" crates/
```

First two commands must return **no matches** (except git history).

`bridged_` is allowed **only** in these three files until [remaining-work.md](remaining-work.md) stream A completes:

- `crates/fpas-vm/src/vm/execute/io/tui/try2/bridged_check_box.rs`
- `crates/fpas-vm/src/vm/execute/io/tui/try2/bridged_radio_button.rs`
- `crates/fpas-vm/src/vm/execute/io/tui/try2/bridged_outline.rs`

Phase 7 sign-off requires zero `bridged_` hits.

## Documentation

- [x] Public spec complete under `docs/pascal/std/tui/` matching [target-api.md](target-api.md) (2026-07-10/11).
- [x] No try-1 `Application.Create*` references in `docs/pascal/std/tui/`.
- [x] `docs/pascal/std/tui/app/vm-bridge.md` describes the try-2 module tree (root dispatch + `try2/`).
- [x] Rust `///` doc links point to current `docs/pascal/std/tui/…` paths (`cargo test -p fpas-vm tui_spec_links`).
- [x] Examples under `examples/pascal/tui/` use try-2 factories.
- [x] [terminal-checklist.md](../pascal/std/tui/terminal-checklist.md) updated.
- [ ] Interim test helper names documented with target `Std.Tui.Test.*` migration — `Test.Click` landed; remaining helpers in [remaining-work.md](remaining-work.md) stream B.

## API completeness (final public spec)

Partial on branch today — see [upstream-mapping.md](upstream-mapping.md#implementation-status-branch-refactortui-try-2-2026-07-09).

- [x] `Application.New`, `Close`, `OpenForTest`, `CloseForTest` — session via try-1 entry points plus `try2.reset()` on close
- [x] `Application.Run`, `Quit`, `ExecView` — try-2 path landed for headless/live smoke; live quit wiring remains a cleanup note in [migration-phases.md](migration-phases.md)
- [x] `Application.MessageBox`
- [x] `Application.RunFileDialog` — live route landed; headless uses a Try-2-local queued adapter because upstream `FileDialog::execute` needs a full `Application`
- [x] `Desktop.Add`
- [x] `Dialog`, `Window`, `Button`, `StaticText`, `InputLine`, `ListBox`, `CheckBox`, `RadioButton`, `Memo`, `TextViewer`
- [x] `MenuBar`, `StatusLine` + setters
- [x] `CM_*` constants for all commands used by IDE and tests (`CM_OPEN`, `CM_ABOUT`, `CM_USER` included)
- [x] `OnCommand`, `OnKey`, `OnMouse`

## Behavioral smoke (manual)

Run in a real terminal:

- [x] Modal: OK and Cancel keys (headless + IDE About)
- [x] Headless mouse: check box and radio button toggle (`tests/tui/events/*_mouse_test.fpas`)
- [x] Menu: pull-down + accelerator (IDE automated + manual 2026-07-09)
- [x] `Application.Quit` from status item (IDE)
- [x] IDE: About, Open, Exit (manual sign-off 2026-07-09)
- [ ] Interactive desktop mouse toggle for checkbox/radio on live window (optional; headless path covered)

## Performance sanity (informal)

- [ ] No full desktop rebuild on `SetText` during `Run` (structural add/remove only mutates tree incrementally)
- [ ] Typing in `InputLine` does not allocate a new desktop each keypress

## Agent / contributor assets

- [x] `.agents/skills/turbo-vision-4-rust/SKILL.md` reflects try-2 architecture (2026-07-11)
- [x] `AGENTS.md` TUI bridge path example matches [rust-layout.md](rust-layout.md)

## Plan closure

- [ ] [migration-phases.md](migration-phases.md) Phase 7 exit criteria (streams A + B + D in [remaining-work.md](remaining-work.md))
- [x] [deletion-checklist.md](deletion-checklist.md) root migration items confirmed (2026-07-11)
- [ ] Archive or remove `docs/refactor-tui-try-2/` after stream A

## Sign-off template

```markdown
## Tui try-2 completion

- Date:
- Branch:
- Upstream pin: turbo-vision v2.0.0
- Docs: docs/pascal/std/tui/ rewritten
- Tests: tests/tui/views/* + apps/ide/tests/
- Removed: ~N LOC bridge (reconcile, bridged, snapshot)
- Manual smoke: pass / notes
```
