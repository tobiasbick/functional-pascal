// [Documentation](../../../../docs/ui/gui/text-and-display/icon.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a glyph from the native icon font.</summary>
    /// <param name="glyph">String containing the icon glyph.</param>
    /// <param name="size">Logical size. Zero selects the native default.</param>
    /// <param name="color">Optional packed straight-alpha RGBA color.</param>
    /// <param name="themeToken">Optional one-based CUI theme token. Zero uses the foreground when color is omitted.</param>
    public unsafe void DisplayIcon(string glyph, float size, uint? color = null, int themeToken = 0)
    {
        CheckFrame();
        ArgumentOutOfRangeException.ThrowIfNegative(themeToken);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(themeToken, ThemeColorCount);
        if (color.HasValue && themeToken != 0)
            throw new ArgumentException("Choose an explicit icon color or a theme token.", nameof(themeToken));
        NativeMethods.DisplayIcon(context, glyph, size, color.GetValueOrDefault(), color.HasValue ? -1 : themeToken);
    }
}
