# Seed field

Use a seed field to vary generated results or keep a seed for reuse.
The gallery uses a dice button and a lock.
Available in **GUI**.

![Seed, dice and lock in the FPAS gallery](images/seed-field.png)

## Example

```pascal
unit Docs.Ui.CreativeControls.SeedField;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Inputs.SeedFields as W;

// Edits an image generation seed.
// ## Parameters
// - `Seed`: Current seed.
// - `Locked`: Whether to keep the current seed.
// ## Returns
// The field with seed and lock proposals.
public function View(Seed: uint; Locked: bool): E.Element of W.Event;
begin
  return W.SeedField of W.Event('seed', new W.Value('Seed', Seed, Locked),
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Field | Default or requirement | Behavior |
| --- | --- | --- |
| `Value.Label` | Required | Sets the caption. Begin with `##` to hide the caption. |
| `Value.Seed` | Required `uint` | Sets the seed displayed by the field. |
| `Value.Locked` | `null` | Hides the lock when `null`. A `true` lock disables the dice. |
| `Options.Display` | Empty | Sets the caption opposite the label. |
| `Options.RandomText` | Empty | Sets the unlocked mode caption. An empty string uses `random`. |
| `Options.FixedText` | Empty | Sets the locked mode caption. An empty string uses `fixed`. |
| `Options.Plain` | `false` | Removes the surrounding well when `true`. |
| `Options.Enabled` | `true` | Accepts pointer and keyboard input when `true`. |

## Interaction

Press Tab to reach the dice and the lock. Press Enter or Space to activate the
focused control. If the seed is locked, the dice cannot receive focus.

The dice proposes a ten-digit seed from a deterministic sequence. The digits
roll into place with motion enabled and read left to right in every layout.
Reducing motion displays the seed immediately and stops a roll in progress.

| Result | Behavior |
| --- | --- |
| `Event.Seed` | Contains the proposed seed. |
| `Event.Locked` | Contains the proposed lock. If the field has no lock, the result is `null`. |
| `Event.Changed` | Is `true` after a roll or a lock toggle. |
| `Event.Rolled` | Is `true` when the dice activates. |

Store the proposed seed and lock in the model to use them when generating a
result.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_seed_field)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Inputs/SeedFields.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Creative/View.fpas)
- [Creative controls](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Inputs/`
  - [SeedField.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Inputs/SeedField.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Inputs/`
  - [NativeMethods.SeedField.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Inputs/NativeMethods.SeedField.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Inputs/`
  - [CuiWindow.SeedField.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Inputs/CuiWindow.SeedField.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/inputs/`
  - [seed_field.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/seed_field.c)
  - [seed_field.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/inputs/seed_field.h)
