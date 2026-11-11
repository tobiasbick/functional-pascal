using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Begins a native centered dialog, including its outgoing transition.</summary>
    /// <param name="id">Stable dialog and panel snapshot identity.</param>
    /// <param name="title">Literal heading.</param>
    /// <param name="description">Optional supporting text.</param>
    /// <param name="width">Preferred width in logical points.</param>
    /// <param name="open">Model-owned visibility.</param>
    /// <param name="enabled">Whether the modal admits input.</param>
    /// <param name="persistent">Suppresses Escape and scrim dismissal.</param>
    /// <param name="closeButton">Whether to draw the native close button.</param>
    /// <param name="focus">Whether to focus the native close button.</param>
    /// <param name="closeControl">Snapshot identity of the close button when present.</param>
    /// <returns>Draw the body for bit 1 and propose dismissal for bit 2.</returns>
    public int BeginDialog(string id, string title, string description, float width, bool open,
        bool enabled, bool persistent, bool closeButton, bool focus, string closeControl)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(description, nameof(description));
        if (!float.IsFinite(width) || width <= 0) throw new ArgumentOutOfRangeException(nameof(width));
        CheckModalDepth();

        int baseNode = NativeMethods.LastNode(context);
        int result = NativeMethods.DialogBegin(context, Stable(id), title, description.Length == 0 ? null : description, width,
            open ? 1 : 0, enabled ? 1 : 0, persistent ? 1 : 0, closeButton ? 1 : 0, focus ? 1 : 0, out int panel);
        if ((result & 1) != 0)
        {
            TrackModal(id);
            layouts.Add(new(Layout.Dialog, Node: baseNode));
            if (open)
            {
                trackedNodes.Add(id, panel);
                if (closeButton) TrackLastControl(closeControl);
            }
        }
        return result;
    }

    /// <summary>Ends the native dialog and restores the base layout target.</summary>
    public void EndDialog()
    {
        var scope = RequireLayout(Layout.Dialog);
        NativeMethods.DialogEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
