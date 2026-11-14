using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native ApiExplorer widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Endpoints">Endpoints supplied to CUI.</param>
    /// <param name="Selection">Selection supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult ApiExplorer(string Id, CuiDevEndpoint[] Endpoints, int Selection, float Height, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevApiExplorer(context, Stable(Id), PackEndpoint(memory, Endpoints), Endpoints.Length, Selection, Height, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
