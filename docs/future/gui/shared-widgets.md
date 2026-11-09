# Shared widget contracts

Define `Fpas.Ui` as the canonical FPAS declarative widget contract for both
hosts. It contains ordinary CLR data/callback types and no renderer or native
dependency. The TUI supports a documented subset with explicit terminal extensions.

## Common API

- [ ] Define one `Element<TMsg>` tree and one set of constructors under
  `Fpas.Ui.Elements`. Start with the existing TUI vocabulary where appropriate:
  text, divider, button, checkbox, text input, multiline editor, list/selection
  list, table, tabs, progress, row/column, border, scrolling and modal overlays.
- [ ] Specify each widget's stable identity, enabled state, model-owned values,
  selection keys and event-to-message callbacks. Use one canonical spelling and
  value contract rather than separate GUI/TUI variants of a common widget.
- [ ] Keep typed cached subviews and semantic style roles. Themes are explicit
  dependencies where they affect a cached view. Avoid CUI-specific ABI structs
  or terminal escape sequences in shared widget data.
- [ ] Publish the support table by widget and option. Limit initial coverage to
  these contracts rather than promising every CUI component.

## Layout and editing

- [ ] Share content/fill sizing, growth/shrinkage, alignment, clipping and semantic
  spacing concepts. Express fixed cell sizes and GUI logical sizes with explicit
  units; do not silently interpret the same numeric size in incompatible units.
- [ ] Distinguish portable responsive view information from backend-specific
  viewport measurements. A shared view must stay within portable widget/options
  contracts; applications may supply different views with the same model/update.
- [ ] Keep application text, caret and selection in the model, using UTF-16
  document offsets with valid editing boundaries. Map CUI's UTF-8/editor state
  explicitly; keep visual scroll offsets and backend geometry unit-aware.
- [ ] Define common focus, modal scope, keyboard activation, selection, disabled
  items and controlled scrolling behavior. Model-visible changes use messages;
  backends may keep transient hover, animation and equivalent presentation state.

## Explicit extensions

- [ ] Provide typed extension nodes for terminal `CellCanvas`/cell styling and
  GUI image/pixel-vector canvas features. Extension payloads remain CLR data;
  native pointers belong to the binding/host, not to application view trees.
- [ ] Make each backend validate the entire tree and its options before
  presentation. An unsupported node, unit or option reports its control/location
  clearly; it must not disappear silently or become an unrelated substitute.
- [ ] Preserve a useful terminal-only surface without taking a dependency on
  CUI. Graphical-only widgets do not create artificial terminal equivalents.

## Acceptance

- [ ] Reuse one portable view and model/update under both hosts with matching
  model values, selection and message semantics.
- [ ] Verify unsupported combinations, duplicate IDs, invalid keys, document
  positions and size-unit mismatches with targeted diagnostics.
- [ ] Keep backend-specific geometry/pixel/cell expectations out of shared
  semantic assertions. Cover typed extensions in their owning host's tests.

Implement the [GUI host](gui-host.md) and [TUI alignment](tui-alignment.md) against
this contract. Public API changes update all repository consumers together.
