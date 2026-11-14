using Fpas.Ui.Gui.Cui.Widgets.Pickers;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private const int SingleCalendarParts = 45;

    /// <summary>Draws CUI's single-date calendar.</summary>
    /// <param name="id">Stable native identity.</param>
    /// <param name="value">Current date, or null.</param>
    /// <param name="enabled">Whether the calendar accepts input.</param>
    /// <param name="focus">Zero for no request, one for previous page, two for next page or three for the title.</param>
    /// <param name="trackPart">Maps native part codes to tracking IDs, or null.</param>
    /// <returns>The proposed date and visible parts.</returns>
    public unsafe CuiCalendarResult Calendar(string id, DateOnly? value, bool enabled, int focus,
        Func<int, string>? trackPart)
    {
        CheckFrame();
        if (focus is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(focus));
        int* parts = stackalloc int[SingleCalendarParts];
        int* nodes = stackalloc int[SingleCalendarParts];
        int proposed = NativeMethods.Calendar(context, Stable(id), value?.DayNumber ?? -1,
            enabled ? 1 : 0, focus, out int root, trackPart is null ? null : parts, nodes, out int count);
        return new(proposed < 0 ? null : DateOnly.FromDayNumber(proposed),
            TrackCalendarParts(id, root, parts, nodes, count, trackPart));
    }

    private unsafe int[] TrackCalendarParts(string id, int root, int* parts, int* nodes, int count,
        Func<int, string>? trackPart)
    {
        if (trackPart is null) return [];
        TrackControl(id, root);
        for (int i = 0; i < count; ++i) TrackControl(trackPart(parts[i]), nodes[i]);
        return new ReadOnlySpan<int>(parts, count).ToArray();
    }
}
