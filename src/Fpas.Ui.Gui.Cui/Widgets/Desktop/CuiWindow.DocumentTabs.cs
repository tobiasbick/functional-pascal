using Fpas.Ui.Gui.Cui.Widgets.Desktop;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native document tabs with keyed selection, close and move proposals.</summary>
    /// <param name="id">Stable strip identity.</param>
    /// <param name="tabs">Documents in display order.</param>
    /// <param name="selected">Selected index or minus one.</param>
    /// <param name="options">Native interaction options.</param>
    /// <param name="enabled">Whether the strip accepts input.</param>
    /// <param name="focus">Whether to focus the selected document.</param>
    /// <param name="trackingIds">Strip and tab inspection identities, or null.</param>
    /// <returns>The proposed selection, close and move.</returns>
    public unsafe CuiDocumentTabChange DocumentTabs(string id, CuiDocumentTab[] tabs, int selected,
        CuiDocumentTabOptions options, bool enabled, bool focus, string[]? trackingIds)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(tabs);
        if (selected < -1 || selected >= tabs.Length) throw new ArgumentOutOfRangeException(nameof(selected));
        if (trackingIds is not null && trackingIds.Length != tabs.Length + 1)
            throw new ArgumentException("Supply the strip and one identity per tab.", nameof(trackingIds));
        var keys = new HashSet<string>(StringComparer.Ordinal);
        string[] titles = new string[tabs.Length], icons = new string[tabs.Length], tips = new string[tabs.Length];
        int[] dirty = new int[tabs.Length], nodes = new int[tabs.Length + 1];
        for (int i = 0; i < tabs.Length; i++)
        {
            CuiDocumentTab tab = tabs[i];
            if (!keys.Add(tab.Key)) throw new ArgumentException("Document keys must be unique.", nameof(tabs));
            Utf8.Validate(tab.Icon, nameof(tabs));
            Utf8.Validate(tab.Tooltip, nameof(tabs));
            titles[i] = ChromeTitle(tab.Title, tab.Key);
            icons[i] = tab.Icon;
            tips[i] = tab.Tooltip;
            dirty[i] = tab.Dirty ? 1 : 0;
        }
        int flags = (options.Shortcuts ? 1 : 0) | (options.Closable ? 0 : 2)
            | (options.Reorderable ? 0 : 4) | (enabled ? 8 : 0) | (focus ? 16 : 0);
        int close, from, to;
        fixed (int* changed = dirty, parts = nodes)
            NativeMethods.DocumentTabs(context, Stable(id), titles, icons, tips, changed,
                tabs.Length, ref selected, flags, out close, out from, out to, parts);
        TrackChromeParts(trackingIds, nodes);
        return new(selected, close, from, to);
    }
}
