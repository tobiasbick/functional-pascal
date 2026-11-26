// [Documentation](../../../../docs/ui/gui/feedback/grid.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens CUI's native equal-width grid. Close it with EndGroup.</summary>
    /// <param name="columns">Fixed count or responsive cap. Zero uses CUI's automatic count.</param>
    /// <param name="minimum">Minimum logical column width, or zero for a fixed count.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical space between cells.</param>
    /// <param name="clip">Whether to clip content at the grid bounds.</param>
    /// <param name="id">Optional stable ID.</param>
    /// <param name="scroll">None, vertical or both axes, in that order.</param>
    public void BeginGrid(int columns, float minimum, float padding, float gap, bool clip = false,
        string? id = null, int scroll = 0)
    {
        CheckLayoutOptions(padding, gap);
        ArgumentOutOfRangeException.ThrowIfNegative(columns);
        if (!float.IsFinite(minimum) || minimum < 0) throw new ArgumentOutOfRangeException(nameof(minimum));
        if (scroll is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(scroll));
        int node = NativeMethods.GridBegin(context, columns, minimum, padding, gap,
            clip ? 1 : 0, id is null ? null : Stable(id), scroll);
        RegisterBox(Layout.Group, node, scrolling: scroll != 0);
    }
}
