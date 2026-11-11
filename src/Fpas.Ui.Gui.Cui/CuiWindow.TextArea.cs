namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native multiline editor with a model-owned value.</summary>
    /// <param name="id">Stable editor identity and snapshot key.</param>
    /// <param name="value">Current multiline text with LF line endings.</param>
    /// <param name="options">Presentation and capacity.</param>
    /// <param name="height">Preferred logical height. Negative fills the remaining space.</param>
    /// <param name="clearable">Show a clear button while editable and nonempty.</param>
    /// <param name="enabled">Whether the editor and its actions accept input.</param>
    /// <param name="focus">Whether to request editor focus.</param>
    /// <param name="track">Whether to track the controls in this frame.</param>
    /// <param name="clearControl">Snapshot identity of the clear button when visible.</param>
    /// <returns>The resulting value, submission and clear-button state.</returns>
    public CuiTextFieldResult TextArea(string id, string value, CuiEditorOptions options,
        float height, bool clearable, bool enabled, bool focus, bool track, string clearControl)
        => DrawEditor(EditorKind.TextArea, id, value, options, height, false, clearable,
            enabled, focus, track, clearControl, null);
}
