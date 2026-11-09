# TUI alignment

Depends on [shared widget contracts](../../../src/Fpas.Ui/README.md), the [MVU core](../../../src/Fpas.Mvu/README.md)
and the [GUI host](gui-host.md). Adapt `Fpas.Tui` to the common contracts while
retaining its terminal renderer and supported behavior.

## Migration

- [ ] Replace the TUI's copies of common widget/model/message types with
  `Fpas.Ui`/`Fpas.Mvu` contracts. Keep one owner for common constructors and
  transitions rather than compatibility aliases and duplicate widget APIs.
- [ ] Adapt cell layout, theme resolution, keyboard/pointer routing, control
  descriptors and cached subviews to the common tree and explicit size units.
- [ ] Preserve terminal editing/selection, disabled item navigation, tabs,
  controlled scroll views, modal focus, scoped shortcuts and resize handling
  according to the shared semantic contract.
- [ ] Keep `CellCanvas`, terminal cell/glyph styling, terminal input handling
  and terminal-specific snapshots in the terminal extension surface.
- [ ] Keep terminal layout, geometry reuse and painting native to the TUI.
  Shared contracts do not require it to call CUI or use GUI measurement/rendering.
- [ ] Run `Fpas.Ui.Validation.Prepare` with the terminal profile before
  presentation. Check terminal-specific limits against the resolved view and
  retain its cache after successful presentation. Verify that supported cell
  extensions keep their behavior and incompatible graphical options produce
  diagnostics.
- [ ] Update affected apps, examples, FPAS tests and C#/F# callers in the same
  migration. Public CLR APIs may be reorganized; FPAS language syntax stays intact.

## Acceptance

- [ ] Run the existing TUI widget, session, input, theme, editor and headless
  contracts against the migrated API. Check actual behavior rather than preserving
  obsolete namespace/name assertions.
- [ ] Retain responsive `CellCanvas`, viewport and controlled-scroll behavior,
  including terminal geometry and Unicode cell-width rules.
- [ ] Run one portable widget view with the same model/update/effects under TUI
  and GUI; compare messages, values, selection and focus semantics.
- [ ] Prove terminal-only build, headless tests and execution work without CUI
  assets or a native GUI toolchain. The dependency graph contains no GUI binding.
- [ ] Update the [TUI reference](../../../src/Fpas.Tui/README.md) and affected
  examples with the implemented contracts and canonical API.

Visual identity across terminal and pixels is not an acceptance criterion.
Shared semantics and explicit backend limitations are.
