using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native TestExplorer widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Entries">Entries supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult TestExplorer(string Id, CuiDevTestItem[] Entries, float Height, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevTestExplorer(context, Stable(Id), PackTestItem(memory, Entries), Entries.Length, Height, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
