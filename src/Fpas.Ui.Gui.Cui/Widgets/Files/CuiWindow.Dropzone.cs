// [Documentation](../../../../docs/ui/gui/capture/dropzone.md)
using Fpas.Ui.Gui.Cui.Widgets.Files;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's dropzone and reads accepted files before the frame can change.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="title">Literal title, or null for the native default.</param>
    /// <param name="description">Literal description, or null for the native default.</param>
    /// <param name="accept">Semicolon-separated extensions, or empty for any type.</param>
    /// <param name="icon">CUI glyph, or null for the native default.</param>
    /// <param name="height">Tall target height in logical points, or zero for the default.</param>
    /// <param name="compact">Whether to draw a compact row.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to track the native bounds.</param>
    /// <param name="widthKind">Native width kind. Content and fill both select CUI fill.</param>
    /// <param name="width">Fixed logical width or percentage of the available width.</param>
    /// <returns>The dropped paths or browse request.</returns>
    public CuiDropzoneResult Dropzone(string id, string? title, string? description, string accept, string? icon,
        float height, bool compact, bool enabled, bool focus, bool track, int widthKind = 1, float width = 0)
    {
        CheckFrame();
        if (title is not null) Utf8.Validate(title, nameof(title));
        if (description is not null) Utf8.Validate(description, nameof(description));
        Utf8.Validate(accept, nameof(accept));
        if (icon is not null) Utf8.Validate(icon, nameof(icon));
        if (!float.IsFinite(height) || height < 0) throw new ArgumentOutOfRangeException(nameof(height));
        CheckExtent(widthKind, width, nameof(width));
        int result = NativeMethods.Dropzone(context, Stable(id), title, description,
            Optional(accept), icon, height, widthKind, width, compact ? 1 : 0, enabled ? 1 : 0,
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
