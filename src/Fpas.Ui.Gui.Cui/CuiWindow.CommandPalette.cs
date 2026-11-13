using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>One CUI palette entry.</summary>
/// <param name="Title">Searchable caption.</param>
/// <param name="Icon">Optional glyph.</param>
/// <param name="Group">Optional group heading.</param>
/// <param name="Shortcut">Optional shortcut hint.</param>
public readonly record struct CuiPaletteCommand(string Title, string Icon, string Group, string Shortcut);

/// <summary>Palette interaction and the indices of rendered command rows.</summary>
/// <param name="Open">Proposed visibility.</param>
/// <param name="Selected">Selected command, or minus one.</param>
/// <param name="Shown">Indices present in the filtered view.</param>
public readonly record struct CuiPaletteResult(bool Open, int Selected, int[] Shown);

public sealed partial class CuiWindow
{
    /// <summary>Reports the native primary+K shortcut when an editor has no keyboard claim.</summary>
    /// <returns>Whether the shortcut was pressed.</returns>
    public bool CommandPaletteShortcut()
    {
        CheckFrame();
        return NativeMethods.PaletteShortcut(context) != 0;
    }

    /// <summary>Draws CUI's searchable modal and returns the chosen command and visibility.</summary>
    /// <param name="id">Stable palette identity.</param>
    /// <param name="open">Current model visibility.</param>
    /// <param name="enabled">Whether the palette accepts input.</param>
    /// <param name="commands">Searchable entries in display order.</param>
    /// <param name="queryId">Inspection identity for the search field.</param>
    /// <param name="rowId">Maps command indices to inspection identities.</param>
    /// <returns>Proposed visibility, selection and filtered indices.</returns>
    public unsafe CuiPaletteResult CommandPalette(string id, bool open, bool enabled, CuiPaletteCommand[] commands, string queryId, Func<int, string> rowId)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(commands);
        int count = commands.Length;
        var titles = new string[count];
        var icons = new string[count];
        var groups = new string[count];
        var shortcuts = new string[count];
        for (int i = 0; i < count; i++)
        {
            var cmd = commands[i];
            Utf8.Validate(cmd.Title, nameof(commands));
            Utf8.Validate(cmd.Icon, nameof(commands));
            Utf8.Validate(cmd.Group, nameof(commands));
            Utf8.Validate(cmd.Shortcut, nameof(commands));
            titles[i] = cmd.Title;
            icons[i] = cmd.Icon;
            groups[i] = cmd.Group;
            shortcuts[i] = cmd.Shortcut;
        }

        // Even an empty command set needs a non-null pointer to report the search field.
        var rows = open ? new int[Math.Max(1, count)] : [];
        int visible = open ? 1 : 0;
        uint owner;
        int selected, node, query;
        fixed (int* parts = rows)
            selected = NativeMethods.CommandPalette(context, Stable(id), ref visible, enabled ? 1 : 0,
                titles, icons, groups, shortcuts, count, out owner, out node, out query, parts);
        TrackModal(id, owner);
        if (!open) return new(visible != 0, selected, []);

        TrackControl(id, node);
        TrackControl(queryId, query);
        List<int> shown = [];
        for (int i = 0; i < count; i++)
        {
            if (rows[i] < 0) continue;
            TrackControl(rowId(i), rows[i]);
            shown.Add(i);
        }
        return new(visible != 0, selected, shown.ToArray());
    }
}
