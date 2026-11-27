// [Documentation](../../../../docs/ui/gui/social-and-media/typing-indicator.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's typing dots with a sender avatar.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="name">Sender name, or null for the anonymous indicator.</param>
    public void TypingIndicator(string id, string? name)
    {
        CheckFrame();
        if (name is not null) Utf8.Validate(name, nameof(name));
        NativeMethods.TypingIndicator(context, Stable(id), name);
    }
}
