using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>The kind of a native menu row.</summary>
public enum CuiMenuEntryKind
{
    /// <summary>A command.</summary>
    Action,
    /// <summary>A section caption.</summary>
    Label,
    /// <summary>A separator.</summary>
    Separator
}

/// <summary>A literal, keyed native menu row.</summary>
/// <param name="Kind">Command, label or separator.</param>
/// <param name="Key">Stable command key; ignored for noninteractive rows.</param>
/// <param name="Label">Literal caption.</param>
/// <param name="Icon">Optional icon-font glyph.</param>
/// <param name="Shortcut">Display-only shortcut hint.</param>
/// <param name="Enabled">Whether the command can be chosen.</param>
/// <param name="Danger">Use the native danger color.</param>
/// <param name="Checked">Show a check mark.</param>
public sealed record CuiMenuEntry(CuiMenuEntryKind Kind, string Key, string Label, string Icon = "",
    string Shortcut = "", bool Enabled = true, bool Danger = false, bool Checked = false);

/// <summary>The native menu's current interaction result.</summary>
/// <param name="Choice">Chosen entry index, or minus one.</param>
/// <param name="Open">Whether the popup accepts input.</param>
/// <param name="Highlighted">Highlighted entry index, or minus one.</param>
public readonly record struct CuiMenuResult(int Choice, bool Open, int Highlighted);

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI dropdown with literal captions and stable command identities.</summary>
    /// <param name="id">Stable trigger identity.</param>
    /// <param name="label">Literal trigger caption and tooltip.</param>
    /// <param name="icon">Optional trigger glyph; icon-only defaults to ellipsis.</param>
    /// <param name="entries">Ordered commands, labels and separators.</param>
    /// <param name="variant">Native button variant ordinal.</param>
    /// <param name="placement">Native popup placement ordinal.</param>
    /// <param name="width">Logical popup width, or zero for content sizing.</param>
    /// <param name="iconOnly">Whether the trigger is an icon button.</param>
    /// <param name="enabled">Whether the menu accepts input.</param>
    /// <param name="focus">Whether to request trigger focus.</param>
    /// <param name="trackingIds">Trigger followed by one identity per entry, or null.</param>
    /// <returns>The chosen entry, visibility and highlight.</returns>
    public CuiMenuResult Menu(string id, string label, string icon, CuiMenuEntry[] entries,
        int variant, int placement, float width, bool iconOnly, bool enabled, bool focus, string[]? trackingIds)
        => DrawMenu(id, label, icon, entries, variant, placement, width, iconOnly, enabled, focus, trackingIds, false);

    /// <summary>Attaches CUI's pointer-positioned menu to the last component without changing its layout.</summary>
    /// <param name="id">Stable menu identity.</param>
    /// <param name="entries">The same commands, labels and separators as a dropdown.</param>
    /// <param name="enabled">Whether this menu accepts input; target behavior remains intact.</param>
    /// <param name="trackingIds">Target followed by one identity per entry, or null.</param>
    /// <returns>The chosen entry, visibility and highlight.</returns>
    public CuiMenuResult ContextMenu(string id, CuiMenuEntry[] entries, bool enabled, string[]? trackingIds)
        => DrawMenu(id, "", "", entries, 0, 0, 0, false, enabled, false, trackingIds, true);

    private unsafe CuiMenuResult DrawMenu(string id, string label, string icon, CuiMenuEntry[] entries,
        int variant, int placement, float width, bool iconOnly, bool enabled, bool focus,
        string[]? trackingIds, bool contextMenu, UserMenuTrigger? user = null)
    {
        CheckFrame();
        if (contextMenu && NativeMethods.LastNode(context) < 0)
            throw new InvalidOperationException("Draw the context menu target first.");
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        ArgumentNullException.ThrowIfNull(entries);
        if (variant is < 0 or 3 or > 7) throw new ArgumentOutOfRangeException(nameof(variant));
        if (placement is < 0 or > 9) throw new ArgumentOutOfRangeException(nameof(placement));
        if (!float.IsFinite(width) || width < 0) throw new ArgumentOutOfRangeException(nameof(width));
        if (trackingIds is not null && trackingIds.Length != entries.Length + 1)
            throw new ArgumentException("Supply the trigger and one identity per menu entry.", nameof(trackingIds));
        string[] keys = new string[entries.Length], labels = new string[entries.Length],
            icons = new string[entries.Length], shortcuts = new string[entries.Length];
        int[] flags = new int[entries.Length], nodes = new int[entries.Length + 1];
        var unique = new HashSet<string>(StringComparer.Ordinal);
        for (int i = 0; i < entries.Length; i++)
        {
            CuiMenuEntry entry = entries[i];
            ArgumentNullException.ThrowIfNull(entry);
            if (!Enum.IsDefined(entry.Kind)) throw new ArgumentException("Unknown menu row kind.", nameof(entries));
            if (entry.Kind == CuiMenuEntryKind.Action && !unique.Add(entry.Key))
                throw new ArgumentException("Menu command keys must be unique.", nameof(entries));
            keys[i] = entry.Kind == CuiMenuEntryKind.Action ? Stable(entry.Key) : "";
            labels[i] = entry.Label; icons[i] = entry.Icon; shortcuts[i] = entry.Shortcut;
            Utf8.Validate(labels[i], nameof(entries)); Utf8.Validate(icons[i], nameof(entries));
            Utf8.Validate(shortcuts[i], nameof(entries));
            flags[i] = (int)entry.Kind | (entry.Enabled ? 0 : 4) | (entry.Danger ? 8 : 0) | (entry.Checked ? 16 : 0);
        }
        int choice, open, highlighted;
        uint owner = 0;
        string key = Stable(id);
        fixed (int* bits = flags, parts = nodes)
        {
            choice = user is null ? NativeMethods.Menu(context, key, label, icon.Length == 0 ? null : icon,
                keys, labels, icons, shortcuts, bits, entries.Length, variant, placement, width,
                iconOnly ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, contextMenu ? 1 : 0, parts, out open, out highlighted, out owner)
                : NativeMethods.UserMenu(context, key, label, user.Detail.Length == 0 ? null : user.Detail,
                    user.Status.Length == 0 ? null : user.Status, keys, labels, icons, shortcuts, bits,
                    entries.Length, placement, width, user.Compact ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0,
                    parts, out open, out highlighted);
        }
        if (contextMenu) TrackModal(id, owner);
        if (trackingIds is not null)
            for (int i = 0; i < nodes.Length; i++)
                if (nodes[i] >= 0)
                {
                    if (contextMenu && i == 0) trackedNodes.Add(trackingIds[i], nodes[i]);
                    else TrackControl(trackingIds[i], nodes[i]);
                }
        return new CuiMenuResult(choice, open != 0, highlighted);
    }
}
