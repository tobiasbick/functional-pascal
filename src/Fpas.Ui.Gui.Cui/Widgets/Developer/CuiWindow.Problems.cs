// [Documentation](../../../../docs/ui/gui/developer-tools/problems-panel.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Problems widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Diagnostics">Diagnostics supplied to CUI.</param>
    /// <param name="Height">Logical height, zero for the native default, or negative to fill the parent.</param>
    /// <param name="HideToolbar">Hide Toolbar supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to focus the diagnostics list.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Problems(string Id, CuiDevDiagnostic[] Diagnostics, float Height, bool HideToolbar, int WidthKind,
        float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (!float.IsFinite(Height)) throw new ArgumentOutOfRangeException(nameof(Height));
        foreach (var item in Diagnostics)
        {
            if (item.Severity is < 0 or > 3 || item.Line < 0 || item.Column < 0)
                throw new ArgumentOutOfRangeException(nameof(Diagnostics));
            Utf8.Validate(item.Message, nameof(Diagnostics));
            if (item.File is { } file) Utf8.Validate(file, nameof(Diagnostics));
            if (item.Source is { } source) Utf8.Validate(source, nameof(Diagnostics));
            if (item.Code is { } code) Utf8.Validate(code, nameof(Diagnostics));
        }
        using var memory = new ChartMemory();
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevProblems(context, Stable(Id), PackDiagnostic(memory, Diagnostics), Diagnostics.Length, Height,
            HideToolbar ? 1 : 0, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
