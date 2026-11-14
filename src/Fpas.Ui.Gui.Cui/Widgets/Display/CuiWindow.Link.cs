using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native inline link.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="enabled">Whether to accept activation.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to retain the link bounds.</param>
    /// <returns>Whether CUI activated the link.</returns>
    public bool Link(string id, string label, bool enabled, bool focus, bool track)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        if (string.IsNullOrWhiteSpace(label)) throw new ArgumentException("A link needs a caption.", nameof(label));
        int activated = NativeMethods.Link(context, Stable(id), label, enabled ? 1 : 0, focus ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return activated != 0;
    }
}
