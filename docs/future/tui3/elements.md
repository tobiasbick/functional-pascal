# Std.Tui3 elements

A `TuiElement` is an immutable description of UI for one frame. Applications build trees
in `View`; they never retain element identity across frames as live objects.

## Element as data

Elements are records or data-carrying enums. Constructors return values. Children are
ordered arrays of elements. There is no public `Destroy`.

Conceptual families:

| Family | Examples | Role |
| --- | --- | --- |
| Chrome | `Desktop`, `Window`, `Dialog`, `MenuBar`, `StatusLine` | Turbo Vision look |
| Layout | `Row`, `Column`, `Grid`, `Form`, `Stack`, `Spacer` | Composition |
| Controls | `Label`, `Button`, `Input`, `CheckBox`, `List`, `Scroll` | Interaction |
| Custom | Deferred until the built-in tree and paint boundary pass Phase 0 | Escape hatch |
| Empty | `None` | Conditional slots |

Sketch:

```pascal
function View(Model: AppModel): TuiElement;
begin
  return Tui.Desktop(Model.Focus, [
    Tui.MenuBar([...]),
    Tui.Window('Editor', [
      Tui.Column([
        Tui.Label('Name'),
        Tui.Input(NameControl, Model.Name, Model.NameCaret, NameChangedAction),
        Tui.Button(SaveControl, 'Save', SaveAction)
      ])
    ]),
    Tui.StatusLine(Model.Status),
    if Model.ConfirmOpen then
      Tui.Dialog('Confirm', [
        Tui.Label('Save changes?'),
        Tui.Row([
          Tui.Button(ConfirmYesControl, 'Yes', ConfirmYesAction),
          Tui.Button(ConfirmNoControl, 'No', ConfirmNoAction)
        ])
      ])
    else
      Tui.None
  ])
end;
```

## Keys

List-like children may carry a stable `Key` (string or integer) so logical items can be compared in
tests or a later internal optimization. A key is not a focus identity. Every interactive node has a
`TuiControlId`, and control ids must be unique in one rendered tree. Duplicate control ids are a
runtime diagnostic because they make focus and routed payloads ambiguous.

## Actions on interactive nodes

Interactive elements store a unique source control id and an application action id, not an
application message value:

```pascal
Tui.Button(Id: TuiControlId; Text: string; Action: TuiAction): TuiElement
Tui.Input(
  Id: TuiControlId;
  Text: string;
  Caret: integer;
  ChangeAction: TuiAction
): TuiElement
```

Clicks become `TuiMsg.Action(Source, Action)`. Controlled input and list changes use dedicated
messages that include the source, action, and complete next state. Action ids may repeat when
several controls represent the same intent; control ids may not. The rule is: **data in the
message, not a widget callback**.

## Controlled controls

Input text, caret position, selection, check state, list selection, focus, and scroll offsets come
from the application model on every `View` call. Routing computes a proposed next value and sends it
to `Update`; it does not keep a second authoritative widget state between frames.

## Turbo Vision chrome

Chrome elements paint TV-like frames and layout slots. They are not ownership roots.

| Element | Look / behavior |
| --- | --- |
| `Desktop` | Full application background; hosts windows and overlays. |
| `Window` | Titled bordered frame; content inset by one cell. |
| `Dialog` | Titled bordered frame; when present in the tree, input focuses its subtree (modal-as-data). |
| `MenuBar` | Top menu strip descriptions. |
| `StatusLine` | Bottom hint / shortcut strip. |

Modal behavior is **structural**: if `View` returns a dialog overlay, the runtime routes
input to that subtree. Closing the dialog means the next `View` omits it after `Update`
clears `Model.ConfirmOpen`.

## Layout elements vs layout engine

`Row`, `Column`, `Grid`, `Form`, and `Stack` are element constructors. The layout engine
interprets them with the size-policy rules in [layout.md](layout.md). There is no separate
live layout handle API.

## Custom paint

Custom paint is deferred until the Phase 0 spike fixes the working-surface ownership and proves the
cost of passing tree values. If added later, it receives a frame-scoped clipped canvas capability;
it must not expose the mutable surface as an ordinary copied value, call `Update`, or mutate the
model.

## Explicit non-goals

- Public retained trees with parent pointers.
- `AsView` / `AsContainer` conversions.
- Per-widget `OnAttach` / `OnDetach` application hooks.
- Reparenting APIs.

Subtree identity for hit-testing exists only for the duration of one laid-out frame.
