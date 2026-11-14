using Fpas.Ui.Gui.Cui.Widgets.Motion;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native number roll.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal display text.</param>
    /// <param name="options">Native effect settings.</param>
    public void NumberRoll(string id, string text, CuiMotionText options)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        var native = MotionOptions(options);
        NativeMethods.NumberRoll(context, Stable(id), text, in native);
    }
}
