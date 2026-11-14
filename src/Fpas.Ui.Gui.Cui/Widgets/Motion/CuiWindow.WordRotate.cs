using Fpas.Ui.Gui.Cui.Widgets.Motion;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native word rotate.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="words">Words in native order.</param>
    /// <param name="interval">Seconds per word, or zero.</param>
    /// <param name="options">Native effect settings.</param>
    public void WordRotate(string id, string[] words, float interval, CuiMotionText options)
    {
        CheckFrame();
        foreach (string word in words) Utf8.Validate(word, nameof(words));
        CheckFeedbackNumber(interval, nameof(interval));
        var native = MotionOptions(options);
        NativeMethods.WordRotate(context, Stable(id), words, words.Length, interval, in native);
    }
}
