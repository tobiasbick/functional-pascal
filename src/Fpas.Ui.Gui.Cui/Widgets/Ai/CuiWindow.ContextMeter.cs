using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native meter and its usage breakdown.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="used">Used context.</param>
    /// <param name="limit">Context capacity.</param>
    /// <param name="parts">Usage breakdown.</param>
    /// <param name="unit">Usage unit.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    public unsafe void ContextMeter(string id, float used, float limit, CuiVizItem[] parts, string unit, int flags)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        NativeMethods.ContextMeter(context, Stable(id), used, limit, memory.Items(parts), parts.Length, OptionalAi(unit), flags);
    }
}
