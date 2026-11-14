using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native copy button and clipboard feedback.</summary>
    /// <param name="id">Logical identity passed to the native widget.</param>
    /// <param name="text">Literal text sent to the clipboard provider on activation.</param>
    /// <param name="enabled">Whether the control accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Bit 1 for a submitted request, bit 2 for visible feedback, and bit 4 for provider availability.</returns>
    public int CopyButton(string id, string text, bool enabled, bool focus)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        return NativeMethods.CopyButton(context, Stable(id), text, enabled ? 1 : 0, focus ? 1 : 0);
    }
}
