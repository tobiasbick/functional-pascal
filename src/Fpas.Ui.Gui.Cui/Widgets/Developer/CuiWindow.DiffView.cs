// [Documentation](../../../../docs/ui/gui/code/diff-viewer.md)
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Reads a page of rows from the most recently presented diff view.</summary>
    /// <param name="id">The same stable identity supplied to DiffView.</param>
    /// <param name="first">Zero-based index of the first displayed row to copy.</param>
    /// <param name="capacity">Maximum rows to copy. Zero reads the total alone.</param>
    /// <returns>A detached page and the displayed total. An absent widget has zero rows.</returns>
    public unsafe CuiDiffRows ReadDiffRows(string id, int first, int capacity)
    {
        CheckPresentedGeometry();
        ArgumentOutOfRangeException.ThrowIfNegative(first);
        ArgumentOutOfRangeException.ThrowIfNegative(capacity);
        string token = Stable(id);
        int total = NativeMethods.DiffViewRows(context, token, null, 0, 0);
        int count = Math.Min(capacity, Math.Max(0, total - first));
        if (count == 0) return new(total, []);
        var rows = new CuiDiffRow[count];
        fixed (CuiDiffRow* data = rows) NativeMethods.DiffViewRows(context, token, data, first, count);
        return new(total, rows);
    }

    /// <summary>Draws the native DiffView widget.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="before">Previous file contents.</param>
    /// <param name="after">Updated file contents.</param>
    /// <param name="current">Selected change, or minus one.</param>
    /// <param name="language">Native syntax language.</param>
    /// <param name="oldTitle">Optional caption for the previous file.</param>
    /// <param name="newTitle">Optional caption for the updated file.</param>
    /// <param name="contextLines">Unchanged lines retained around changes.</param>
    /// <param name="widthKind">Native extent kind.</param>
    /// <param name="width">Extent value for fixed and percentage widths.</param>
    /// <param name="height">Logical height, or the native auto/fill sentinel.</param>
    /// <param name="size">Logical font size, or zero for the default.</param>
    /// <param name="splitMinWidth">Minimum logical width for split layout, or zero for the native default.</param>
    /// <param name="flags">Widget-specific input and rendering flags.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public CuiDiffViewEvent? DiffView(string id, string before, string after, int current, string language, string oldTitle, string newTitle, int contextLines, int widthKind, float width, float height, float size, float splitMinWidth, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(splitMinWidth, nameof(splitMinWidth));
        int changed = NativeMethods.DiffView(context, Stable(id), before, after, current, language,
            OptionalAi(oldTitle), OptionalAi(newTitle), contextLines, widthKind, width, height, size, splitMinWidth, flags,
            out var result);
        return changed == 0 ? null : new(result.Current, result.Count, result.Inline != 0,
            result.Added, result.Removed, result.Folds, result.Hidden,
            result.Approximate != 0, result.CurrentChanged != 0, result.Recomputed != 0);
    }
}
