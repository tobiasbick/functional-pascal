using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native initials or a decoded avatar image with optional presence.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="name">Name used for initials.</param>
    /// <param name="path">Optional model-owned image file path.</param>
    /// <param name="status">Optional presence name.</param>
    /// <param name="size">Logical diameter, or zero for the native default.</param>
    /// <param name="square">Whether to draw a rounded square.</param>
    /// <param name="track">Whether to retain the avatar bounds.</param>
    /// <returns>Whether CUI decoded the supplied image.</returns>
    public bool Avatar(string id, string name, string? path, string status, float size, bool square, bool track)
    {
        CheckFrame();
        Utf8.Validate(name, nameof(name));
        if (path is not null) Utf8.Validate(path, nameof(path));
        Utf8.Validate(status, nameof(status));
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        int image = NativeMethods.Avatar(context, Stable(id), name, path, Optional(status), size, square ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return image != 0;
    }
}
