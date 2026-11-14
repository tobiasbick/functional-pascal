using Fpas.Ui.Gui.Cui.Widgets.Display;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native Markdown widget.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal source text.</param>
    /// <param name="widthKind">Native extent kind.</param>
    /// <param name="width">Extent value for fixed and percentage widths.</param>
    /// <param name="size">Logical font size, or zero for the default.</param>
    /// <param name="flags">Widget-specific input and rendering flags.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public unsafe CuiMarkdownEvent Markdown(string id, string text, int widthKind, float width, float size, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        nint link = NativeMethods.Markdown(context, Stable(id), text, widthKind, width, size, flags, out int offset, out int line, out int check,
            out nint hovered, out int hoverChanged);
        return new(System.Runtime.InteropServices.Marshal.PtrToStringUTF8(link), offset, line, check != 0,
            System.Runtime.InteropServices.Marshal.PtrToStringUTF8(hovered), hoverChanged != 0);
    }
}
