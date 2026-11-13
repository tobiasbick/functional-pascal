using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>The native picker closure and its visible parts.</summary>
/// <param name="Closed">The picker was dismissed or confirmed this frame.</param>
/// <param name="Path">Chosen path, or null on dismissal.</param>
/// <param name="Parts">Availability of the dialog and its controls.</param>
public readonly record struct CuiFilePickerResult(bool Closed, string? Path, bool[] Parts);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native modal file picker.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="open">Requested visibility.</param>
    /// <param name="enabled">Accepts modal input.</param>
    /// <param name="mode">Open, save or directory selection.</param>
    /// <param name="title">Optional title.</param>
    /// <param name="start">Initial folder.</param>
    /// <param name="name">Initial save name.</param>
    /// <param name="filter">Optional file patterns.</param>
    /// <param name="filterLabel">Optional caption for the patterns.</param>
    /// <param name="list">Optional synchronous folder listing.</param>
    /// <param name="part">Maps visible native parts to tracking identities.</param>
    /// <returns>Native modal closure and optional path.</returns>
    public unsafe CuiFilePickerResult FilePicker(string id, bool open, bool enabled, int mode, string title,
        string start, string name, string filter, string filterLabel, Func<string, bool, CuiFileEntry[]?>? list,
        Func<int, string>? part)
    {
        CheckFrame();
        if (mode is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(mode));
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(start, nameof(start));
        Utf8.Validate(name, nameof(name));
        Utf8.Validate(filter, nameof(filter));
        Utf8.Validate(filterLabel, nameof(filterLabel));
        if (Utf8.Encoding.GetByteCount(start) >= 1024 || Utf8.Encoding.GetByteCount(name) >= 256)
            throw new ArgumentException("The start folder or save name exceeds the native buffer.");
        using var source = new CuiFileSource(list);
        byte* path = stackalloc byte[1024];
        int* nodes = stackalloc int[5];
        int status = NativeMethods.FilePicker(context, Stable(id), open ? 1 : 0, enabled ? 1 : 0, mode,
            Optional(title), Optional(start), Optional(name), Optional(filter), Optional(filterLabel),
            source.Callback, source.User, path, 1024, nodes);
        var parts = new bool[5];
        for (int i = 0; i < parts.Length; ++i)
        {
            parts[i] = nodes[i] >= 0;
            if (parts[i] && part is not null) TrackControl(part(i), nodes[i]);
        }
        return new(status != 0, status == 2 ? Marshal.PtrToStringUTF8((nint)path) : null, parts);
    }
}
