using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native DisplayIcon operation.</summary>
    /// <param name="glyph">glyph for the native widget.</param>
    /// <param name="size">size for the native widget.</param>
    /// <param name="color">color for the native widget.</param>
    public unsafe void DisplayIcon(string glyph, float size, uint color)
    {
        CheckFrame();
        NativeMethods.DisplayIcon(context, glyph, size, color);
    }
}
