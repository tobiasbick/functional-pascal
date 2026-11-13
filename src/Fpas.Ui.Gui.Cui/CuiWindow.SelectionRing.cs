using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native SelectionRing operation.</summary>
    /// <param name="tone">tone supplied to the native operation.</param>
    /// <param name="rgba">rgba supplied to the native operation.</param>
    /// <param name="name">name supplied to the native operation.</param>
    /// <param name="radius">radius supplied to the native operation.</param>
    public unsafe void SelectionRing(int tone, uint rgba, string? name, float radius)
    {
        CheckFrame();
        NativeMethods.SelectionRing(context, tone, rgba, name, radius);
    }
}
