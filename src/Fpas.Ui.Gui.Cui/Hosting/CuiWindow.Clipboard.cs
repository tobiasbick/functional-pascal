using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Submits literal text to the window's system clipboard provider.</summary>
    /// <param name="text">Unicode clipboard text.</param>
    public void WriteClipboardText(string text)
    {
        CheckOpen();
        Utf8.Validate(text, nameof(text));
        NativeMethods.WriteClipboardText(context, text);
    }

    /// <summary>Copies the current system clipboard text into a managed string.</summary>
    /// <returns>The clipboard text, or an empty string if unavailable.</returns>
    public string ReadClipboardText()
    {
        CheckOpen();
        return Marshal.PtrToStringUTF8(NativeMethods.ReadClipboardText(context)) ?? string.Empty;
    }
}
