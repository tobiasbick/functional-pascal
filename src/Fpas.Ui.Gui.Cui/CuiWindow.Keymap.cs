using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Keymap widget.</summary>
    /// <param name="Id">Id supplied to CUI.</param>
    /// <param name="Bindings">Bindings supplied to CUI.</param>
    /// <param name="Categories">Categories supplied to CUI.</param>
    /// <param name="Modifiers">Modifiers supplied to CUI.</param>
    /// <param name="HideFunctionRow">Hide Function Row supplied to CUI.</param>
    /// <param name="WidthKind">Width Kind supplied to CUI.</param>
    /// <param name="Width">Width supplied to CUI.</param>
    /// <param name="Enabled">Enabled supplied to CUI.</param>
    /// <returns>Values proposed by CUI.</returns>
    public unsafe CuiDevResult Keymap(string Id, CuiDevBinding[] Bindings, string[] Categories, int Modifiers, bool HideFunctionRow, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        using var memory = new ChartMemory();
        NativeMethods.DevKeymap(context, Stable(Id), PackBinding(memory, Bindings), Bindings.Length, memory.Labels(Categories), Categories.Length, Modifiers, HideFunctionRow ? 1 : 0, WidthKind, Width, Enabled ? 1 : 0, out var result);
        return result.ToManaged();
    }
}
