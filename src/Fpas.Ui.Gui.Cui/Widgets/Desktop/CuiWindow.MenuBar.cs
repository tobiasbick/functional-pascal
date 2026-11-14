using Fpas.Ui.Gui.Cui.Widgets.Overlays;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a native menu bar.</summary>
    /// <param name="id">Stable bar ID.</param>
    /// <param name="border">Whether to draw a bottom hairline.</param>
    /// <param name="enabled">Whether the titles accept pointer and keyboard input.</param>
    public void BeginMenuBar(string id, bool border, bool enabled)
    {
        CheckFrame();
        NativeMethods.MenuBarBegin(context, Stable(id), border ? 1 : 0, enabled ? 1 : 0);
    }

    /// <summary>Draws a title with its native popup entries.</summary>
    /// <param name="id">Stable title key in this bar.</param>
    /// <param name="label">Visible title.</param>
    /// <param name="mnemonic">Optional ASCII mnemonic.</param>
    /// <param name="width">Popup width in logical points, or zero for automatic sizing.</param>
    /// <param name="enabled">Whether this menu is enabled.</param>
    /// <param name="entries">Commands, captions and separators.</param>
    /// <param name="trackingIds">Trigger and entry identities for inspection, or null.</param>
    /// <returns>Activation and popup state.</returns>
    public unsafe CuiMenuResult MenuBarMenu(string id, string label, string mnemonic, float width,
        bool enabled, CuiMenuEntry[] entries, string[]? trackingIds)
    {
        CheckFrame();
        CheckFeedbackNumber(width, nameof(width));
        Utf8.Validate(mnemonic, nameof(mnemonic));
        if (mnemonic.Length > 1 || mnemonic.Length == 1 && !char.IsAsciiLetterOrDigit(mnemonic[0]))
            throw new ArgumentException("Expected an ASCII letter or digit.", nameof(mnemonic));
        var (keys, labels, icons, shortcuts, flags) = PackMenuEntries(entries, trackingIds);
        int[] nodes = new int[entries.Length + 1];
        int choice, open, highlighted;
        fixed (int* bits = flags, parts = nodes)
            choice = NativeMethods.MenuBarMenu(context, ChromeTitle(label, id),
                mnemonic.Length == 0 ? 0 : mnemonic[0], width, enabled ? 1 : 0,
                keys, labels, icons, shortcuts, bits, entries.Length, parts, out open, out highlighted);
        TrackChromeParts(trackingIds, nodes);
        return new(choice, open != 0, highlighted);
    }

    /// <summary>Closes the native menu bar.</summary>
    public void EndMenuBar()
    {
        CheckFrame();
        NativeMethods.MenuBarEnd(context);
    }

    private static string ChromeTitle(string label, string key)
    {
        Utf8.Validate(label, nameof(label));
        if (label.Contains("##", StringComparison.Ordinal))
            throw new ArgumentException("A chrome caption cannot contain the native key delimiter.", nameof(label));
        return label + Stable(key);
    }

    private void TrackChromeParts(string[]? trackingIds, int[] nodes)
    {
        if (trackingIds is null) return;
        for (int i = 0; i < nodes.Length; i++)
            if (nodes[i] >= 0) TrackControl(trackingIds[i], nodes[i]);
    }
}
