using Fpas.Ui.Gui.Cui.Widgets.Files;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's uploadable avatar.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="name">Name used for initials.</param>
    /// <param name="path">Image path, or null to display initials.</param>
    /// <param name="size">Logical diameter, or zero for the default.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to track the native bounds.</param>
    /// <returns>The displayed image state and any native file action.</returns>
    public CuiAvatarUploadResult AvatarUpload(string id, string name, string? path, float size,
        bool enabled, bool focus, bool track)
    {
        CheckFrame();
        Utf8.Validate(name, nameof(name));
        if (path is not null) Utf8.Validate(path, nameof(path));
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        int result = NativeMethods.AvatarUpload(context, Stable(id), name, path, size, enabled ? 1 : 0,
            focus ? 1 : 0, out int node, out int hasImage);
        if (track) TrackControl(id, node);
        return new(result == 1 ? DroppedPath(0) : null, result < 0, hasImage != 0);
    }
}
