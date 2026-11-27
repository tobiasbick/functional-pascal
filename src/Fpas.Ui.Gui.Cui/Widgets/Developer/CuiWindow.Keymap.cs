// [Documentation](../../../../docs/ui/gui/developer-tools/keymap.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Keymap widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Bindings">Bindings supplied to CUI.</param>
    /// <param name="Categories">Categories supplied to CUI.</param>
    /// <param name="Modifiers">Modifier mask, or null to keep the native selection.</param>
    /// <param name="HideFunctionRow">Hide Function Row supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <param name="Focus">Whether to focus the modifier selector.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Keymap(string Id, CuiDevBinding[] Bindings, string[] Categories, int? Modifiers,
        bool HideFunctionRow, int WidthKind, float Width, bool Enabled, bool Focus = false)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        if (Modifiers is < 0 or > 15) throw new ArgumentOutOfRangeException(nameof(Modifiers));
        foreach (string category in Categories) Utf8.Validate(category, nameof(Categories));
        foreach (var binding in Bindings)
        {
            Utf8.Validate(binding.Action, nameof(Bindings));
            if (binding.Modifiers is < 0 or > 15 || binding.Key is < 0 or >= 68 || binding.Category < 0)
                throw new ArgumentOutOfRangeException(nameof(Bindings));
        }
        using var memory = new ChartMemory();
        int flags = (Enabled ? 1 : 0) | (Focus ? 2 : 0);
        NativeMethods.DevKeymap(context, Stable(Id), PackBinding(memory, Bindings), Bindings.Length, memory.Labels(Categories),
            Categories.Length, Modifiers ?? -1, HideFunctionRow ? 1 : 0, WidthKind, Width, flags, out var result);
        return result.ToManaged();
    }
}
