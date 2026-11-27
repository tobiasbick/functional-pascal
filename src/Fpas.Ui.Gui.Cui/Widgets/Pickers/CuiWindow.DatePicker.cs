// [Documentation](../../../../docs/ui/gui/inputs/date-picker.md)
using Fpas.Ui.Gui.Cui.Widgets.Pickers;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's date field and calendar popup.</summary>
    /// <param name="id">Stable native identity.</param>
    /// <param name="value">Current date, or null.</param>
    /// <param name="label">Optional caption above the field.</param>
    /// <param name="placeholder">Text for an empty field, or null for CUI's default.</param>
    /// <param name="widthKind">Content, fill, fixed or percentage in that order.</param>
    /// <param name="width">Fixed logical width or percentage, when selected.</param>
    /// <param name="enabled">Whether the picker accepts input.</param>
    /// <param name="focus">Whether to request trigger focus.</param>
    /// <param name="trackPart">Maps native part codes to tracking IDs, or null.</param>
    /// <returns>The proposed date and visible popup parts.</returns>
    public unsafe CuiCalendarResult DatePicker(string id, DateOnly? value, string label, string? placeholder,
        int widthKind, float width, bool enabled, bool focus, Func<int, string>? trackPart)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        if (placeholder is not null) Utf8.Validate(placeholder, nameof(placeholder));
        CheckExtent(widthKind, width, nameof(width));
        int* parts = stackalloc int[SingleCalendarParts];
        int* nodes = stackalloc int[SingleCalendarParts];
        int proposed = NativeMethods.DatePicker(context, Stable(id), value?.DayNumber ?? -1,
            Optional(label), placeholder, widthKind, width, enabled ? 1 : 0, focus ? 1 : 0,
            out int root, parts, nodes, out int count);
        return new(proposed < 0 ? null : DateOnly.FromDayNumber(proposed),
            TrackCalendarParts(id, root, parts, nodes, count, trackPart));
    }
}
