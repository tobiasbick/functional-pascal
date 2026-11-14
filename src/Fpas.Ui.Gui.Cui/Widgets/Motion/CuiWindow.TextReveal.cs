using Fpas.Ui.Gui.Cui.Widgets.Motion;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native text reveal.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal display text.</param>
    /// <param name="replay">Revision that restarts the effect.</param>
    /// <param name="options">Native effect settings.</param>
    public void TextReveal(string id, string text, int replay, CuiMotionText options)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        var native = MotionOptions(options);
        NativeMethods.TextReveal(context, Stable(id), text, replay, in native);
    }
}
