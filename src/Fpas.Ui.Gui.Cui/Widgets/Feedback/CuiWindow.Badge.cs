using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native badge.</summary>
    /// <param name="text">Literal caption.</param>
    /// <param name="tone">Native semantic tone.</param>
    /// <param name="dot">Whether to draw the leading status dot.</param>
    /// <param name="solid">Whether to use solid fill.</param>
    /// <param name="icon">Optional CUI glyph.</param>
    /// <param name="size">Native control size.</param>
    public void Badge(string text, int tone, bool dot, bool solid, string icon, int size)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(icon, nameof(icon));
        if (tone is < 0 or > 9) throw new ArgumentOutOfRangeException(nameof(tone));
        if (size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        NativeMethods.Badge(context, text, tone, dot ? 1 : 0, solid ? 1 : 0, Optional(icon), size);
    }
}
