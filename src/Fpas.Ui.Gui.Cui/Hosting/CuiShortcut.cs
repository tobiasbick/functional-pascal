namespace Fpas.Ui.Gui.Cui;

/// <summary>An application shortcut using native key/modifier ordinals.</summary>
/// <param name="Key">Native key ordinal, excluding None.</param>
/// <param name="Modifiers">Exact native Shift/Control/Alt/Super bits.</param>
/// <param name="Enabled">Whether the binding is active in this view.</param>
/// <param name="Repeat">Whether native repeats may activate the binding in later frames.</param>
/// <param name="AllowWhileEditing">Whether an unconsumed combination may run while editing text.</param>
public readonly record struct CuiShortcut(int Key, int Modifiers = 0, bool Enabled = true,
    bool Repeat = false, bool AllowWhileEditing = false);
