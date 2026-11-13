using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native DisplayKeycap operation.</summary>
    /// <param name="keys">keys for the native widget.</param>
    public unsafe void DisplayKeycap(string keys)
    {
        CheckFrame();
        NativeMethods.DisplayKeycap(context, keys);
    }
}
