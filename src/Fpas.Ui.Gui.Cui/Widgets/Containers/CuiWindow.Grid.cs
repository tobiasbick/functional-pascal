using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens CUI's native equal-width grid. Close it with EndGroup.</summary>
    /// <param name="columns">Maximum number of columns.</param>
    /// <param name="minimum">Minimum logical column width, or zero for a fixed count.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical space between cells.</param>
    public void BeginGrid(int columns, float minimum, float padding, float gap)
    {
        CheckLayoutOptions(padding, gap);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(columns);
        if (!float.IsFinite(minimum) || minimum < 0) throw new ArgumentOutOfRangeException(nameof(minimum));
        layouts.Add(new(Layout.Group, Node: NativeMethods.GridBegin(context, columns, minimum, padding, gap)));
    }
}
