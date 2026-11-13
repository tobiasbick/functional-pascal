using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Current native folder and selection.</summary>
/// <param name="Folder">Current folder.</param>
/// <param name="Selected">Selected entry, or null.</param>
/// <param name="Picked">Activated file, or null.</param>
/// <param name="Error">The folder was unreadable.</param>
/// <param name="Parts">Availability of browser parts.</param>
public readonly record struct CuiFileBrowserResult(string Folder, string? Selected, string? Picked, bool Error, bool[] Parts);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's browser for the supplied folder.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="folder">Current folder.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Preferred width.</param>
    /// <param name="heightKind">Native height kind.</param>
    /// <param name="height">Preferred height.</param>
    /// <param name="filter">Optional file patterns.</param>
    /// <param name="places">Optional sidebar destinations.</param>
    /// <param name="list">Optional synchronous folder listing.</param>
    /// <param name="maximum">Entry limit, or zero.</param>
    /// <param name="revision">Data revision that requests a reload.</param>
    /// <param name="flags">Native input and presentation bits.</param>
    /// <param name="focus">Requests focus on the file list.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>Native folder and selection.</returns>
    public unsafe CuiFileBrowserResult FileBrowser(string id, string folder, int widthKind, float width,
        int heightKind, float height, string filter, string[] places, Func<string, bool, CuiFileEntry[]?>? list,
        int maximum, int revision, int flags, bool focus, Func<int, string>? part)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        if (places.Length > 6 || maximum is < 0 or > 100_000)
            throw new ArgumentOutOfRangeException(nameof(maximum));
        Utf8.Validate(filter, nameof(filter));
        foreach (string place in places) Utf8.Validate(place, nameof(places));
        using var source = new CuiFileSource(list);
        Span<byte> path = stackalloc byte[1024];
        CuiFileSource.WritePath(folder, path);
        int* nodes = stackalloc int[7];
        fixed (byte* buffer = path)
        {
            int flagsOut = NativeMethods.FileBrowser(context, Stable(id), buffer, path.Length, widthKind, width,
                heightKind, height, Optional(filter), places, places.Length, source.Callback, source.User,
                maximum, revision, flags, focus ? 1 : 0, out nint selected, out nint picked, nodes);
            var parts = new bool[7];
            for (int i = 0; i < parts.Length; ++i)
            {
                parts[i] = nodes[i] >= 0;
                if (parts[i] && part is not null) TrackControl(part(i), nodes[i]);
            }
            return new(Marshal.PtrToStringUTF8((nint)buffer)!, Marshal.PtrToStringUTF8(selected),
                Marshal.PtrToStringUTF8(picked), (flagsOut & 2) != 0, parts);
        }
    }
}
