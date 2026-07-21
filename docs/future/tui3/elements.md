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
| Custom | `Canvas` / paint callback element | Escape hatch |
| Empty | `None` | Conditional slots |

Sketch:

```pascal
function View(Model: AppModel): TuiElement;
begin
  return Tui.Desktop([
    Tui.MenuBar([...]),
    Tui.Window('Editor', [
      Tui.Column([
        Tui.Label('Name'),
        Tui.Input(Model.Name, NameAction),
        Tui.Button('Save', SaveAction)
      ])
    ]),
    Tui.StatusLine(Model.Status),
    if Model.ConfirmOpen then
      Tui.Dialog('Confirm', [
        Tui.Label('Save changes?'),
        Tui.Row([
          Tui.Button('Yes', ConfirmYesAction),
          Tui.Button('No', ConfirmNoAction)
        ])
      ])
    else
      Tui.None
  ])
end;
```

## Keys

List-like children may carry a stable `Key` (string or integer) so focus and selection can
follow logical items across rebuilds. Keys are optional in v1 when the application stores
selection indexes in the model explicitly.

## Actions on interactive nodes

Interactive elements store a `TuiAction` id, not an application message value:

```pascal
Tui.Button(Text: string; Action: TuiAction): TuiElement
Tui.Input(Text: string; ChangeAction: TuiAction): TuiElement
```

Text changes and clicks become `TuiMsg.Action` (and for inputs, the runtime includes the
new text in a dedicated message variant or pairs action id with a payload record). The
exact payload shape is fixed in [api-surface.md](api-surface.md); the rule is: **data in
the message, not a widget callback**.

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

A custom element supplies intrinsic size and a paint function over `TuiCanvas` for one
frame. The paint function must not call `Update` or mutate the model. Prefer built-in
controls when they suffice.

## Explicit non-goals

- Public retained trees with parent pointers.
- `AsView` / `AsContainer` conversions.
- Per-widget `OnAttach` / `OnDetach` application hooks.
- Reparenting APIs.

Subtree identity for hit-testing exists only for the duration of one laid-out frame.
