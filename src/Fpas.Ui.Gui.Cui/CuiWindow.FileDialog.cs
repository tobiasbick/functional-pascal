using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens the platform file dialog outside an active frame.</summary>
    /// <param name="title">Literal title, or empty for the platform default.</param>
    /// <param name="filterName">Display name for the extension filter.</param>
    /// <param name="accept">Semicolon-separated extensions, or empty for any type.</param>
    /// <returns>The selected UTF-8 path, or null on cancellation or a filter mismatch.</returns>
    public unsafe string? OpenFile(string title, string filterName, string accept)
    {
        CheckOpen();
        if (inFrame) throw new InvalidOperationException("Open the file dialog between frames.");
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(filterName, nameof(filterName));
        Utf8.Validate(accept, nameof(accept));
        const int capacity = 8192;
        byte* path = stackalloc byte[capacity];
        return NativeMethods.OpenFile(app, Optional(title), Optional(filterName), Optional(accept), path, capacity) != 0
            ? Marshal.PtrToStringUTF8((nint)path)
            : null;
    }
}
