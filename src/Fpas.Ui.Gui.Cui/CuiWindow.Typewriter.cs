using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Current typewriter progress at the end of native presentation.</summary>
/// <param name="Done">Whether all characters are visible.</param>
/// <param name="Completed">Whether this frame completed the current text or replay.</param>
public readonly record struct CuiTypewriterResult(bool Done, bool Completed);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native typewriter.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal display text.</param>
    /// <param name="replay">Revision that restarts the effect.</param>
    /// <param name="options">Native effect settings.</param>
    /// <returns>Completion state and transition.</returns>
    public CuiTypewriterResult Typewriter(string id, string text, int replay, CuiMotionText options)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        var native = MotionOptions(options);
        int state = NativeMethods.Typewriter(context, Stable(id), text, replay, in native);
        return new((state & 1) != 0, (state & 2) != 0);
    }
}
