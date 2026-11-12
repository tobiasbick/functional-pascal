using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A native date range selection and its visible parts.</summary>
/// <param name="Start">Start day number, or minus one.</param>
/// <param name="End">End day number, or minus one.</param>
/// <param name="Changed">Whether CUI proposed a changed selection.</param>
/// <param name="Open">Whether a picker popup is open.</param>
/// <param name="VisibleParts">Day numbers and codes for visible navigation and presets.</param>
public readonly record struct CuiDateRangeResult(int Start, int End, bool Changed, bool Open, int[] VisibleParts);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's inline range calendar.</summary>
    /// <param name="id">Stable native identity.</param>
    /// <param name="start">Start day number, or minus one.</param>
    /// <param name="end">End day number, or minus one.</param>
    /// <param name="months">One or two visible months.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="trackPart">Maps native part codes to tracking IDs, or null.</param>
    /// <returns>The native proposal and visible parts.</returns>
    public unsafe CuiDateRangeResult RangeCalendar(string id, int start, int end, int months,
        bool enabled, Func<int, string>? trackPart)
    {
        CheckFrame();
        ValidateDateRange(start, end);
        if (months is < 1 or > 2) throw new ArgumentOutOfRangeException(nameof(months));
        int* parts = stackalloc int[72];
        int* nodes = stackalloc int[72];
        bool changed = NativeMethods.RangeCalendar(context, Stable(id), ref start, ref end,
            months, enabled ? 1 : 0, out int root, parts, nodes, out int count) != 0;
        return new(start, end, changed, false, TrackCalendarParts(id, root, parts, nodes, count, trackPart));
    }

    /// <summary>Draws CUI's native date range picker.</summary>
    /// <param name="id">Stable trigger identity.</param>
    /// <param name="start">Start day number, or minus one.</param>
    /// <param name="end">End day number, or minus one.</param>
    /// <param name="label">Optional label above the field.</param>
    /// <param name="placeholder">Optional caption for an empty range.</param>
    /// <param name="width">Logical width, or zero for CUI's default.</param>
    /// <param name="presets">Whether to show native relative presets.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to request trigger focus.</param>
    /// <param name="trackPart">Maps native part codes to tracking IDs, or null.</param>
    /// <returns>The native proposal, popup state and visible parts.</returns>
    public unsafe CuiDateRangeResult RangePicker(string id, int start, int end, string label,
        string placeholder, float width, bool presets, bool enabled, bool focus, Func<int, string>? trackPart)
    {
        CheckFrame();
        ValidateDateRange(start, end);
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(placeholder, nameof(placeholder));
        if (!float.IsFinite(width) || width < 0) throw new ArgumentOutOfRangeException(nameof(width));
        int* parts = stackalloc int[72];
        int* nodes = stackalloc int[72];
        bool changed = NativeMethods.RangePicker(context, Stable(id), ref start, ref end,
            Optional(label), Optional(placeholder), width, presets ? 1 : 0, enabled ? 1 : 0,
            focus ? 1 : 0, out int root, parts, nodes, out int count, out int open) != 0;
        return new(start, end, changed, open != 0, TrackCalendarParts(id, root, parts, nodes, count, trackPart));
    }

    private static void ValidateDateRange(int start, int end)
    {
        if (start < -1 || start > DateOnly.MaxValue.DayNumber)
            throw new ArgumentOutOfRangeException(nameof(start));
        if (end < -1 || end > DateOnly.MaxValue.DayNumber || (end >= 0 && (start < 0 || start > end)))
            throw new ArgumentOutOfRangeException(nameof(end));
    }

}
