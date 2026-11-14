using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Terminal widget.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="output">Cumulative ANSI output.</param>
    /// <param name="revision">Revision used to reset the scrollback.</param>
    /// <param name="prompt">Prompt shown at the input line.</param>
    /// <param name="widthKind">Native extent kind.</param>
    /// <param name="width">Extent value for fixed and percentage widths.</param>
    /// <param name="height">Logical height, or the native auto/fill sentinel.</param>
    /// <param name="size">Logical font size, or zero for the default.</param>
    /// <param name="flags">Widget-specific input and rendering flags.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public unsafe CuiTerminalEvent Terminal(string id, string output, int revision, string prompt, int widthKind, float width, float height, float size, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        nint command = NativeMethods.Terminal(context, Stable(id), output, revision, prompt, widthKind, width, height, size, flags, out int interrupt);
        return new(System.Runtime.InteropServices.Marshal.PtrToStringUTF8(command), interrupt != 0);
    }
}
