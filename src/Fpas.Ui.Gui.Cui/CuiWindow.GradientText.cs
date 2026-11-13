using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native gradient text.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal display text.</param>
    /// <param name="options">Native effect settings.</param>
    /// <param name="to">Second gradient color.</param>
    /// <param name="animate">Whether to animate the gradient.</param>
    public void GradientText(string id, string text, CuiMotionText options, CuiChartColor to, bool animate)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        var native = MotionOptions(options);
        NativeMethods.GradientText(context, Stable(id), text, in native, to.Palette, to.Rgba, animate ? 1 : 0);
    }
}
