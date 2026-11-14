using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native search field with a model-owned value.</summary>
    /// <param name="id">Stable editor identity and snapshot key.</param>
    /// <param name="value">Current single-line search query.</param>
    /// <param name="options">Presentation and capacity.</param>
    /// <param name="enabled">Whether the editor and its actions accept input.</param>
    /// <param name="focus">Whether to request editor focus.</param>
    /// <param name="track">Whether to track the controls in this frame.</param>
    /// <param name="clearControl">Snapshot identity of the clear button when visible.</param>
    /// <returns>The resulting value, submission and clear-button state.</returns>
    public CuiTextFieldResult SearchField(string id, string value, CuiEditorOptions options,
        bool enabled, bool focus, bool track, string clearControl)
        => DrawEditor(EditorKind.SearchField, id, value, options, 0, false, true,
            enabled, focus, track, clearControl, null);
}
