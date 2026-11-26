// [Documentation](../../../../docs/ui/gui/text-and-display/text.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native text role with optional reading direction and alignment.</summary>
    /// <param name="text">Literal UTF-8-compatible text.</param>
    /// <param name="role">Body zero, headings one through four, label five, secondary six, muted seven, or code eight.</param>
    /// <param name="direction">Inherit, left-to-right, right-to-left, or automatic, in native order.</param>
    /// <param name="alignment">Start, center, right, left, or end, in native order.</param>
    public void DisplayText(string text, int role, int direction = 0, int alignment = 0)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        if (role is < 0 or > 8) throw new ArgumentOutOfRangeException(nameof(role));
        if (direction is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(direction));
        if (alignment is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(alignment));
        NativeMethods.DisplayText(context, text, role, direction, alignment);
    }
}
