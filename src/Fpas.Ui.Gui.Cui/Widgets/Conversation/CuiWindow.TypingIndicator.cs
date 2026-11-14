using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's typing dots with a sender avatar.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="name">Sender name.</param>
    public void TypingIndicator(string id, string name)
    {
        CheckFrame();
        Utf8.Validate(name, nameof(name));
        NativeMethods.TypingIndicator(context, Stable(id), name);
    }
}
