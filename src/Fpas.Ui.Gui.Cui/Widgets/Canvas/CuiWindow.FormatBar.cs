using Fpas.Ui.Gui.Cui.Interop.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native FormatBar operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="format">format supplied to the native operation.</param>
    /// <param name="styles">styles supplied to the native operation.</param>
    /// <param name="colors">colors supplied to the native operation.</param>
    /// <param name="footer">footer supplied to the native operation.</param>
    /// <param name="keys">keys supplied to the native operation.</param>
    /// <param name="question">question supplied to the native operation.</param>
    /// <param name="flags">flags supplied to the native operation.</param>
    /// <returns>The native result.</returns>
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
