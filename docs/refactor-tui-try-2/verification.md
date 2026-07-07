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
rg "TurboVisionObject|pending_reconcile|FPAS_TV_COMMAND_OFFSET|bridged_" crates/
rg "TuiCreateDialog|TuiAddChild|TuiTestSetDialogResult" crates/
rg "Application\.CreateDialog|Application\.AddChild|Command\.Quit" docs/pascal/std/tui/
```

All must return **no matches** (except git history).

## Documentation

- [ ] Public spec complete under `docs/pascal/std/tui/` matching [target-api.md](target-api.md).
- [ ] No try-1 `Application.Create*` references in `docs/pascal/`.
- [ ] `docs/pascal/std/tui/app/vm-bridge.md` describes ≤15 bridge modules.
- [ ] Rust `///` doc links point to current `docs/pascal/std/tui/…` paths.
- [ ] Examples under `examples/pascal/tui/` compile and run.
- [ ] [terminal-checklist.md](../pascal/std/tui/terminal-checklist.md) updated.

## API completeness (final public spec)

Partial on branch today — see [upstream-mapping.md](upstream-mapping.md#implementation-status-branch-refactortui-try-2-2026-07-07).

- [ ] `Application.New`, `Close`, `Size`, `OpenForTest`, `CloseForTest` — **partial:** `New`/session via try-1 + `try2.reset()`
- [ ] `Application.Run`, `Quit`, `ExecView` — **partial:** try-2 path landed for headless smoke
- [ ] `Application.MessageBox`, `RunFileDialog`
- [ ] `Desktop.Add`
- [ ] `Dialog`, `Window`, `Button`, `StaticText`, `InputLine`, `ListBox`, `CheckBox`, `RadioButton`, `Memo`, `TextViewer`
- [ ] `MenuBar`, `StatusLine` + setters
- [ ] `CM_*` constants for all commands used by IDE and tests
- [ ] `OnCommand`; `OnKey` / `OnMouse` if promised in spec

## Behavioral smoke (manual)

Run in a real terminal:

- [ ] Modal: OK and Cancel keys
- [ ] Mouse: check box and radio button toggle
- [ ] Menu: pull-down + accelerator
- [ ] `Application.Quit` from status item
- [ ] IDE: About, Open, Exit

## Performance sanity (informal)

- [ ] No full desktop rebuild on `SetText` during `Run` (structural add/remove only mutates tree incrementally)
- [ ] Typing in `InputLine` does not allocate a new desktop each keypress

## Agent / contributor assets

- [ ] `.agents/skills/turbo-vision-4-rust/SKILL.md` reflects try-2 architecture
- [ ] `AGENTS.md` TUI bridge path example matches [rust-layout.md](rust-layout.md)

## Plan closure

- [ ] [migration-phases.md](migration-phases.md) checkboxes complete through phase 7
- [ ] [deletion-checklist.md](deletion-checklist.md) items confirmed
- [ ] Archive or remove `docs/refactor-tui-try-2/`

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
