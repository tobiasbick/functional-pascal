using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native NetworkWaterfall widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Requests">Requests supplied to CUI.</param>
    /// <param name="Selection">Selection supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="DomLoaded">Dom Loaded supplied to CUI.</param>
    /// <param name="Loaded">Loaded supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult NetworkWaterfall(string Id, CuiDevRequest[] Requests, int Selection, float Height, double DomLoaded, double Loaded, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevNetworkWaterfall(context, Stable(Id), PackRequest(memory, Requests), Requests.Length, Selection, Height, DomLoaded, Loaded, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
