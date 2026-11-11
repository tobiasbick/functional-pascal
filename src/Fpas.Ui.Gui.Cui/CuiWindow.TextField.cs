namespace Fpas.Ui.Gui.Cui;

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

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's single-line editor using a detached UTF-8 buffer.</summary>
    /// <param name="id">Stable editor identity and snapshot key.</param>
    /// <param name="value">Current model-owned text.</param>
    /// <param name="options">Presentation and capacity.</param>
    /// <param name="enabled">Whether the editor and its actions accept input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="track">Whether to track the controls in this frame.</param>
    /// <param name="clearControl">Snapshot key of the optional clear button.</param>
    /// <param name="revealControl">Snapshot key of the optional password reveal button.</param>
    /// <returns>The resulting value, submission and affordance state.</returns>
    public CuiTextFieldResult TextField(string id, string value, CuiTextFieldOptions options,
        bool enabled, bool focus, bool track, string clearControl, string revealControl)
    {
        ArgumentNullException.ThrowIfNull(options);
        var presentation = new CuiEditorOptions(options.Label, options.Placeholder, options.Icon, options.Hint,
            options.Error, options.Prefix, options.Suffix, options.Size, options.ReadOnly, options.MaxUtf8Bytes);
        return DrawEditor(EditorKind.TextField, id, value, presentation, 0, options.Password, options.Clearable,
            enabled, focus, track, clearControl, revealControl);
    }
}
