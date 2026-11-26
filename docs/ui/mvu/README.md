# Model–View–Update

The GUI and TUI hosts use `Fpas.Ui.Mvu`. The model holds application state. A view reads the model and builds widgets. Input produces a typed message. The update function returns the next model and any commands for the host to execute.

```pascal
unit Docs.Ui.Mvu.Counter;

uses Fpas.Ui.Mvu.Application as Mvu;

public type Msg = variant
  Increment;
  Close;
end variant;

// Applies a counter action or asks the host to close.
public function Update(Count: int; Message: Msg): Mvu.Transition of (int, Msg);
begin
  return case Message of
    when Msg.Increment: new Mvu.Transition of (int, Msg)(Count + 1, []);
    when Msg.Close: new Mvu.Transition of (int, Msg)(Count, [Mvu.Exit of Msg()]);
  end case;
end function;

end unit;
```

## Commands and effects

Return an empty command array for a state-only change. Use `Focus` to request focus by widget ID and `Exit` to close. Run a [background effect](runtime.md#background-effects) with `StartBackground` and apply its messages in the update function.

## Composing larger views

Give each independently updated feature its own model, messages, update function and view. The parent owns the child model and wraps child messages in a case of its own message type. Use `Fpas.Ui.Composition.Map` to translate the child view's messages into that case. The terminal gallery’s profile editor shows this composition in a small application. The GUI gallery uses slices for larger pages.

- [Profile editor](../../../examples/ui/tui/gallery/ProfileSample.fpas) and [parent view](../../../examples/ui/tui/gallery/main.fpas)
- [Runtime, commands and slice helpers](runtime.md)
- [Subview composition](../widgets.md#subview-composition)
- [GUI handbook](../gui/README.md) and [TUI handbook](../tui/README.md)
