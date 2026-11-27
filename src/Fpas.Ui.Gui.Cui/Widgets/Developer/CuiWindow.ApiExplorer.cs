// [Documentation](../../../../docs/ui/gui/more-developer-tools/api-explorer.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native ApiExplorer widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Endpoints">Endpoints supplied to CUI.</param>
    /// <param name="Selection">Selected index, minus one for none, or null for native selection.</param>
    /// <param name="Height">List height in logical units. Zero fits content and a negative value fills the parent.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to focus the endpoint list.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult ApiExplorer(string Id, CuiDevEndpoint[] Endpoints, int? Selection, float Height, int WidthKind, float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (!float.IsFinite(Height)) throw new ArgumentOutOfRangeException(nameof(Height));
        if (Selection is int index && (index < -1 || index >= Endpoints.Length))
            throw new ArgumentOutOfRangeException(nameof(Selection));
        using var memory = new ChartMemory();
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevApiExplorer(context, Stable(Id), PackEndpoint(memory, Endpoints), Endpoints.Length, Selection ?? -2, Height, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
