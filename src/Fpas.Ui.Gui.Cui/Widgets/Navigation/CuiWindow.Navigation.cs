using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a sidebar column to be closed with <see cref="EndGroup"/>.</summary>
    /// <param name="id">Identity shared by the sidebar's destinations.</param>
    /// <param name="hasSelection">Whether to retain the native selection highlight.</param>
    public void BeginSidebar(string id, bool hasSelection)
    {
        string group = Stable(id);
        BeginGroup(false, 0, 2, 0, false);
        if (!hasSelection) NativeMethods.ClearSidebar(context, group);
    }

    /// <summary>Draws a non-interactive native sidebar heading.</summary>
    /// <param name="title">Literal visible heading.</param>
    public void SidebarHeading(string title)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        NativeMethods.SidebarHeading(context, title);
    }

    /// <summary>Draws a native sidebar destination that fills the available width.</summary>
    /// <param name="group">Stable identity shared by the sidebar's destinations.</param>
    /// <param name="id">Stable identity of this destination.</param>
    /// <param name="text">Literal visible caption.</param>
    /// <param name="icon">CUI icon glyph, or empty.</param>
    /// <param name="badge">Literal badge text, or empty.</param>
    /// <param name="enabled">Whether to accept input.</param>
    /// <param name="selected">Whether the destination is active.</param>
    /// <param name="focus">Whether to request focus before drawing.</param>
    /// <returns>Whether the destination was activated.</returns>
    public bool SidebarItem(string group, string id, string text, string icon, string badge,
        bool enabled, bool selected, bool focus)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(badge, nameof(badge));
        return NativeMethods.SidebarItem(context, Stable(group), Stable(id), text,
            icon.Length == 0 ? null : icon, badge.Length == 0 ? null : badge,
            enabled ? 1 : 0, selected ? 1 : 0, focus ? 1 : 0) != 0;
    }
}
