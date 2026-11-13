using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native DisplayEmoji operation.</summary>
    /// <param name="glyph">glyph for the native widget.</param>
    /// <param name="size">size for the native widget.</param>
    public unsafe void DisplayEmoji(string glyph, float size)
    {
        CheckFrame();
        NativeMethods.DisplayEmoji(context, glyph, size);
    }
}
