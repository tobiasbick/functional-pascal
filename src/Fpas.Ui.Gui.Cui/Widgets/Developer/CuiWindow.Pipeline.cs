using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Pipeline widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Stages">Stages supplied to CUI.</param>
    /// <param name="Jobs">Jobs supplied to CUI.</param>
    /// <param name="Selection">Selection supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Pipeline(string Id, string[] Stages, CuiDevJob[] Jobs, int Selection, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevPipeline(context, Stable(Id), memory.Labels(Stages), Stages.Length, PackJob(memory, Jobs), Jobs.Length, Selection, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
