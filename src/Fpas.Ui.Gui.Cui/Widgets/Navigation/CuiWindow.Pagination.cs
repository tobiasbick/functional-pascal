using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native pagination with a model-owned page.</summary>
    /// <param name="id">Stable pager identity.</param>
    /// <param name="page">Zero-based selected page.</param>
    /// <param name="count">Positive page count.</param>
    /// <param name="enabled">Whether the pager accepts input.</param>
    /// <param name="focus">Page index, -1 for Previous, -2 for Next, or -3 for none.</param>
    /// <param name="partId">Maps visible button indices to tracking IDs, or null.</param>
    /// <param name="visible">Rendered page indices with -1 for Previous and -2 for Next.</param>
    /// <returns>The proposed page, or minus one when unchanged.</returns>
    public unsafe int Pagination(string id, int page, int count, bool enabled, int focus,
        Func<int, string>? partId, out int[] visible)
    {
        CheckFrame();
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(count);
        if (page < 0 || page >= count) throw new ArgumentOutOfRangeException(nameof(page));
        if (focus < -3 || focus >= count) throw new ArgumentOutOfRangeException(nameof(focus));
        int* parts = stackalloc int[18];
        int proposal = NativeMethods.Pagination(context, Stable(id), page, count, enabled ? 1 : 0, focus, parts, out int used);
        visible = new int[used];
        for (int index = 0; index < used; index++)
        {
            int key = parts[index * 2];
            visible[index] = key;
            if (partId is not null) TrackControl(partId(key), parts[index * 2 + 1]);
        }
        return proposal;
    }
}
