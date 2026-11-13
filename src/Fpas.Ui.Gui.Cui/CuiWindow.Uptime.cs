using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Uptime widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Services">Services supplied to CUI.</param>
    /// <param name="Year">Year supplied to CUI.</param>
    /// <param name="Month">Month supplied to CUI.</param>
    /// <param name="Day">Day supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Uptime(string Id, CuiDevService[] Services, int Year, int Month, int Day, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevUptime(context, Stable(Id), PackService(memory, Services), Services.Length, Year, Month, Day, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
