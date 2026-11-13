using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Problems widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Diagnostics">Diagnostics supplied to CUI.</param>
    /// <param name="Height">Height supplied to CUI.</param>
    /// <param name="HideToolbar">Hide Toolbar supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Problems(string Id, CuiDevDiagnostic[] Diagnostics, float Height, bool HideToolbar, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevProblems(context, Stable(Id), PackDiagnostic(memory, Diagnostics), Diagnostics.Length, Height, HideToolbar ? 1 : 0, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
