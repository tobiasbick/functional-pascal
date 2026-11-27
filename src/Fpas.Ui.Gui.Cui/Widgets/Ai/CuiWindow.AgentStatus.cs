// [Documentation](../../../../docs/ui/gui/ai/agent-status.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native agent status pill.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Status caption.</param>
    /// <param name="status">Native task state.</param>
    /// <param name="seconds">Elapsed seconds.</param>
    /// <param name="icon">Optional glyph.</param>
    public unsafe void AgentStatus(string id, string label, int status, float seconds, string icon)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        CheckFeedbackNumber(seconds, nameof(seconds));
        if ((double)seconds > int.MaxValue) throw new ArgumentOutOfRangeException(nameof(seconds));
        if (status is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(status));
        NativeMethods.AgentStatus(context, Stable(id), label, status, seconds, OptionalAi(icon));
    }
}
