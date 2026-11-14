using Fpas.Ui.Gui.Cui.Widgets.Motion;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native shimmer text.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal display text.</param>
    /// <param name="options">Native effect settings.</param>
    public void ShimmerText(string id, string text, CuiMotionText options)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        var native = MotionOptions(options);
        NativeMethods.ShimmerText(context, Stable(id), text, in native);
    }
}
