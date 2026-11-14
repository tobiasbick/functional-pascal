using Fpas.Ui.Gui.Cui.Widgets.Pickers;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's date field and calendar popup.</summary>
    /// <param name="id">Stable native identity.</param>
    /// <param name="value">Current date, or null.</param>
    /// <param name="label">Optional caption above the field.</param>
    /// <param name="placeholder">Optional text for an empty field.</param>
    /// <param name="width">Logical width, or zero for the native default.</param>
    /// <param name="enabled">Whether the picker accepts input.</param>
    /// <param name="focus">Whether to request trigger focus.</param>
    /// <param name="trackPart">Maps native part codes to tracking IDs, or null.</param>
    /// <returns>The proposed date and visible popup parts.</returns>
    public unsafe CuiCalendarResult DatePicker(string id, DateOnly? value, string label, string placeholder,
        float width, bool enabled, bool focus, Func<int, string>? trackPart)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(placeholder, nameof(placeholder));
        if (!float.IsFinite(width) || width < 0) throw new ArgumentOutOfRangeException(nameof(width));
        int* parts = stackalloc int[SingleCalendarParts];
        int* nodes = stackalloc int[SingleCalendarParts];
        int proposed = NativeMethods.DatePicker(context, Stable(id), value?.DayNumber ?? -1,
            Optional(label), Optional(placeholder), width, enabled ? 1 : 0, focus ? 1 : 0,
            out int root, parts, nodes, out int count);
        return new(proposed < 0 ? null : DateOnly.FromDayNumber(proposed),
            TrackCalendarParts(id, root, parts, nodes, count, trackPart));
    }
}
