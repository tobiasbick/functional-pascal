using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native divider in a conversation.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Visible caption.</param>
    public void ChatDivider(string id, string label)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        NativeMethods.ChatDivider(context, Stable(id), label);
    }
}
