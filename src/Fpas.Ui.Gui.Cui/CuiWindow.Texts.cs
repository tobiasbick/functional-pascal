using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native DisplayText operation.</summary>
    /// <param name="text">text for the native widget.</param>
    /// <param name="role">role for the native widget.</param>
    public unsafe void DisplayText(string text, int role)
    {
        CheckFrame();
        NativeMethods.DisplayText(context, text, role);
    }
}
