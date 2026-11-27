// [Documentation](../../../../docs/ui/gui/developer-tools/network-waterfall.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native NetworkWaterfall widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Requests">Requests supplied to CUI.</param>
    /// <param name="Selection">Model selection, minus one for none, or null for native selection.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="DomLoaded">Dom Loaded supplied to CUI.</param>
    /// <param name="Loaded">Loaded supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to request keyboard focus.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult NetworkWaterfall(string Id, CuiDevRequest[] Requests, int? Selection, float Height, double DomLoaded, double Loaded, int WidthKind, float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Selection is int row && (row < -1 || row >= Requests.Length))
            throw new ArgumentOutOfRangeException(nameof(Selection));
        using var memory = new ChartMemory();
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevNetworkWaterfall(context, Stable(Id), PackRequest(memory, Requests), Requests.Length, Selection ?? -2, Height, DomLoaded, Loaded, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
