// [Documentation](../../../../docs/ui/gui/actions/user-menu.md)
using Fpas.Ui.Gui.Cui.Widgets.Overlays;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private sealed record UserMenuTrigger(string? Detail, string? Status, bool Compact, string? AvatarPath);

    /// <summary>Draws CUI's account trigger and its native menu.</summary>
    /// <param name="id">Stable trigger identity.</param>
    /// <param name="name">Account display name.</param>
    /// <param name="detail">Text below the name. Null omits the line.</param>
    /// <param name="status">Native avatar status. Null omits the dot.</param>
    /// <param name="compact">Whether the trigger shows the avatar alone.</param>
    /// <param name="entries">Ordered commands and decorations.</param>
    /// <param name="placement">Native popup placement ordinal.</param>
    /// <param name="width">Logical popup width, or zero for CUI's default.</param>
    /// <param name="enabled">Whether the trigger and commands accept input.</param>
    /// <param name="focus">Whether to focus the trigger.</param>
    /// <param name="trackingIds">Trigger and ordered entry identities, or null.</param>
    /// <param name="avatarPath">Optional native image file; null uses initials.</param>
    /// <returns>The choice, popup visibility and highlight.</returns>
    public CuiMenuResult UserMenu(string id, string name, string? detail, string? status, bool compact,
        CuiMenuEntry[] entries, int placement, float width, bool enabled, bool focus, string[]? trackingIds, string? avatarPath = null)
    {
        if (detail is not null) Utf8.Validate(detail, nameof(detail));
        if (status is not null) Utf8.Validate(status, nameof(status));
        if (avatarPath is not null) Utf8.Validate(avatarPath, nameof(avatarPath));
        return DrawMenu(id, name, "", entries, 0, placement, width, false, enabled, focus, trackingIds,
            false, new UserMenuTrigger(detail, status, compact, avatarPath));
    }
}
