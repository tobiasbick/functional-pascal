using Fpas.Ui.Gui.Cui.Widgets.Ai;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native model menu and returns the current or proposed selection.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="models">Available model choices.</param>
    /// <param name="selected">Current model index.</param>
    /// <param name="placement">Native popup placement.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    public unsafe int ModelPicker(string id, CuiModelItem[] models, int selected, int placement, int flags)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        NativeMethods.ModelPicker(context, Stable(id), NativeModels(memory, models), models.Length, &selected, placement, flags);
        return selected;
    }
}
