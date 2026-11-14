using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui;

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
