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
        => SelectFile(CuiFileDialogKind.Open, title, filterName, accept, "");

    /// <summary>Opens CUI's native save chooser without creating or writing the selected file.</summary>
    /// <param name="title">Optional dialog title.</param>
    /// <param name="filterName">Optional name for the native extension filter.</param>
    /// <param name="accept">Semicolon-separated extensions without dots.</param>
    /// <param name="defaultName">Optional proposed filename or path.</param>
    /// <returns>The selected UTF-8 path, or null when CUI returns no selection.</returns>
    public string? SaveFile(string title, string filterName, string accept, string defaultName)
        => SelectFile(CuiFileDialogKind.Save, title, filterName, accept, defaultName);

    /// <summary>Opens CUI's native directory chooser on the owner thread.</summary>
    /// <param name="title">Optional dialog title.</param>
    /// <returns>The selected UTF-8 directory, or null when CUI returns no selection.</returns>
    public string? ChooseFolder(string title) => SelectFile(CuiFileDialogKind.Folder, title, "", "", "");

    private unsafe string? SelectFile(CuiFileDialogKind kind, string title, string filterName, string accept, string defaultName)
    {
        CheckOpen();
        if (inFrame) throw new InvalidOperationException("Open the file dialog between frames.");
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(filterName, nameof(filterName));
        Utf8.Validate(accept, nameof(accept));
        Utf8.Validate(defaultName, nameof(defaultName));
        const int capacity = 8192;
        byte* path = stackalloc byte[capacity];
        return NativeMethods.FileDialog(app, (int)kind, Optional(title), Optional(filterName),
            Optional(accept), Optional(defaultName), path, capacity) != 0
            ? Marshal.PtrToStringUTF8((nint)path)
            : null;
    }
}
