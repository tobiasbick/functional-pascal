// [Documentation](../../../../docs/ui/gui/more-developer-tools/coverage.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Coverage widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Files">Files supplied to CUI.</param>
    /// <param name="Height">Height in logical units. Zero fits the rows up to 360 units. Negative values fill the available height.</param>
    /// <param name="Low">Low supplied to CUI.</param>
    /// <param name="High">High supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to request keyboard focus for the file list.</param>
    /// <returns>The activated file in Action and aggregate line percentage in X.</returns>
    public unsafe CuiDevResult Coverage(string Id, CuiDevCoverageFile[] Files, float Height, float Low, float High, int WidthKind, float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (!float.IsFinite(Height))
            throw new ArgumentOutOfRangeException(nameof(Height));
        using var memory = new ChartMemory();
        var flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevCoverage(context, Stable(Id), PackCoverageFile(memory, Files), Files.Length, Height, Low, High, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
