// [Documentation](../../../../docs/ui/gui/files/terminal.md)
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
    /// <param name="flags">Enabled (1), show the input line (2), and request focus (4).</param>
    /// <param name="byteCapacity">Native text buffer capacity in UTF-8 bytes, at least 64.</param>
    /// <param name="lineCapacity">Native line buffer capacity, at least two.</param>
    /// <returns>Submitted text or an interrupt from the native control.</returns>
    public CuiTerminalEvent Terminal(string id, string output, int revision, string prompt, int widthKind,
        float width, float height, float size, int flags, int byteCapacity = 64 * 1024, int lineCapacity = 1024)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        if (height != -1) CheckFeedbackNumber(height, nameof(height));
        CheckFeedbackNumber(size, nameof(size));
        Utf8.Validate(output, nameof(output));
        Utf8.Validate(prompt, nameof(prompt));
        ArgumentOutOfRangeException.ThrowIfLessThan(byteCapacity, 64);
        ArgumentOutOfRangeException.ThrowIfLessThan(lineCapacity, 2);
        nint command = NativeMethods.Terminal(context, Stable(id), output, revision, prompt, widthKind,
            width, height, size, byteCapacity, lineCapacity, flags, out int interrupt);
        return new(System.Runtime.InteropServices.Marshal.PtrToStringUTF8(command), interrupt != 0);
    }
}
