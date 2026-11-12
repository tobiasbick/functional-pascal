using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>File paths or a browse activation from CUI's dropzone.</summary>
/// <param name="Paths">Accepted paths copied out of the current native frame.</param>
/// <param name="Browse">Whether the user requested a file dialog.</param>
public readonly record struct CuiDropzoneResult(string[] Paths, bool Browse);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's dropzone and reads accepted files before the frame can change.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="title">Literal title, or empty for the native default.</param>
    /// <param name="description">Literal description, or empty for the native default.</param>
    /// <param name="accept">Semicolon-separated extensions, or empty for any type.</param>
    /// <param name="icon">Optional CUI glyph.</param>
    /// <param name="height">Logical height, or zero for the default.</param>
    /// <param name="compact">Whether to draw a compact row.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to track the native bounds.</param>
    /// <returns>The dropped paths or browse request.</returns>
    public CuiDropzoneResult Dropzone(string id, string title, string description, string accept, string icon,
        float height, bool compact, bool enabled, bool focus, bool track)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(description, nameof(description));
        Utf8.Validate(accept, nameof(accept));
        Utf8.Validate(icon, nameof(icon));
        if (!float.IsFinite(height) || height < 0) throw new ArgumentOutOfRangeException(nameof(height));
        int result = NativeMethods.Dropzone(context, Stable(id), Optional(title), Optional(description),
            Optional(accept), Optional(icon), height, compact ? 1 : 0, enabled ? 1 : 0,
            focus ? 1 : 0, out int node);
        if (track) TrackControl(id, node);

        if (result <= 0) return new([], result < 0);
        var paths = new string[result];
        for (int index = 0; index < paths.Length; ++index)
            paths[index] = DroppedPath(index);
        return new(paths, false);
    }

    private string DroppedPath(int index) =>
        Marshal.PtrToStringUTF8(NativeMethods.DroppedFile(context, index))
        ?? throw new InvalidOperationException("CUI reported a file drop without a path.");
}
