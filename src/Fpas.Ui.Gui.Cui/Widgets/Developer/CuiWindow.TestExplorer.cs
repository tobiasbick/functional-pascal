// [Documentation](../../../../docs/ui/gui/more-developer-tools/test-explorer.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native TestExplorer widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Entries">Tests and suites in tree order.</param>
    /// <param name="Height">Logical height, zero for 320, or negative to fill the parent.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Selected and activated item indices, run requests, and test counts.</returns>
    public unsafe CuiDevResult TestExplorer(string Id, CuiDevTestItem[] Entries, float Height, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (!float.IsFinite(Height)) throw new ArgumentOutOfRangeException(nameof(Height));
        int previousDepth = -1;
        foreach (var entry in Entries)
        {
            if (entry.Depth < 0 || entry.Depth > 63 || entry.Depth > previousDepth + 1 ||
                entry.Status is < 0 or > 5 || !float.IsFinite(entry.Duration))
                throw new ArgumentOutOfRangeException(nameof(Entries));
            Utf8.Validate(entry.Name, nameof(Entries));
            if (entry.Message is { } message) Utf8.Validate(message, nameof(Entries));
            previousDepth = entry.Depth;
        }
        using var memory = new ChartMemory();
        NativeMethods.DevTestExplorer(context, Stable(Id), PackTestItem(memory, Entries), Entries.Length, Height, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
