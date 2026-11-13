using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Coverage widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Files">Files supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="Low">Low supplied to CUI.</param>
    /// <param name="High">High supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Coverage(string Id, CuiDevCoverageFile[] Files, float Height, float Low, float High, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevCoverage(context, Stable(Id), PackCoverageFile(memory, Files), Files.Length, Height, Low, High, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
