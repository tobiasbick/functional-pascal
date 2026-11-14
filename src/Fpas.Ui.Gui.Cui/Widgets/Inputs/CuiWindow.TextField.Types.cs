namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>Presentation and capacity of a native single-line editor.</summary>
/// <param name="Label">Optional caption above the field.</param>
/// <param name="Placeholder">Text displayed while empty.</param>
/// <param name="Icon">Optional leading glyph.</param>
/// <param name="Hint">Supporting text below the field.</param>
/// <param name="Error">Validation error that replaces the hint.</param>
/// <param name="Prefix">Muted text before the editable value.</param>
/// <param name="Suffix">Muted text after the editable value.</param>
/// <param name="Size">Native medium, small or large size ordinal.</param>
/// <param name="Password">Mask the text and show the native reveal button.</param>
/// <param name="Clearable">Show a clear button while editable and nonempty.</param>
/// <param name="ReadOnly">Allow focus and selection while preventing edits.</param>
/// <param name="MaxUtf8Bytes">Maximum value size excluding the terminator.</param>
public sealed record CuiTextFieldOptions(string Label, string Placeholder, string Icon, string Hint, string Error,
    string Prefix, string Suffix, int Size, bool Password, bool Clearable, bool ReadOnly, int MaxUtf8Bytes);

/// <summary>One frame's edit and native field affordances.</summary>
/// <param name="Value">Proposed value after native input.</param>
/// <param name="Submitted">Whether Enter submitted the field.</param>
/// <param name="ClearVisible">Whether the clear button was drawn.</param>
/// <param name="RevealVisible">Whether the reveal button was drawn.</param>
/// <param name="Revealed">Whether a password is currently revealed.</param>
public sealed record CuiTextFieldResult(string Value, bool Submitted, bool ClearVisible, bool RevealVisible, bool Revealed);
