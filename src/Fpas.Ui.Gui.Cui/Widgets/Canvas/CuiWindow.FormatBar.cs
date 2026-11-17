using Fpas.Ui.Gui.Cui.Interop.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native formatting bar and writes proposed formatting changes.</summary>
    /// <param name="id">Stable formatting-bar identity.</param>
    /// <param name="format">Current and proposed text style, color, alignment and list settings.</param>
    /// <param name="styles">Named style choices with native font ordinals and logical text sizes.</param>
    /// <param name="colors">Native semantic tone ordinals available in the color menu.</param>
    /// <param name="footer">Optional footer caption.</param>
    /// <param name="keys">Optional footer keyboard hint.</param>
    /// <param name="question">Optional delete-confirmation caption.</param>
    /// <param name="flags">Bits: 1 accepts input, 2 hides delete, 4 hides the additional-actions button.</param>
    /// <returns>Bits: 1 format changed, 2 link requested, 4 delete confirmed, 8 additional actions requested; zero when idle or disabled.</returns>
    public unsafe int FormatBar(string id, ref CuiTextFormat format, CuiTextStyle[] styles, int[] colors, string? footer, string? keys, string? question, int flags)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        var native = Array.ConvertAll(styles, style => new NativeTextStyle
        {
            Name = memory.Text(style.Name), Size = style.Size, Preview = style.Preview, Font = style.Font
        });
        fixed (int* tones = colors)
            return NativeMethods.FormatBar(context, Stable(id), ref format, memory.Copy<NativeTextStyle>(native),
                styles.Length, tones, colors.Length, footer, keys, question, flags);
    }
}
