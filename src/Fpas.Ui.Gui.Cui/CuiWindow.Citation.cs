using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native citation with its preview and reports activation.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="number">Displayed fallback number.</param>
    /// <param name="source">Optional preview source.</param>
    /// <param name="flags">Native input and appearance flags.</param>
    public unsafe bool Citation(string id, int number, CuiSourceItem? source, int flags)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        NativeSource item = source is null ? default : NativeCitation(memory, source);
        return NativeMethods.Citation(context, Stable(id), number, source is null ? null : &item, flags) != 0;
    }
}
