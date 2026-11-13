using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Frame output from the native tree table.</summary>
/// <param name="Activated">Activated node, or minus one.</param>
/// <param name="Load">Node indices requesting children.</param>
public readonly record struct CuiTreeTableEvents(int Activated, int[] Load);

public sealed partial class CuiWindow
{
    private sealed record TreeOutput(nint Handle, uint[] Flags, int[] State);

    /// <summary>Draws a native tree cell for composition inside a table.</summary>
    /// <param name="id">Stable cell and inspection identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="icon">Optional CUI glyph.</param>
    /// <param name="depth">Indentation level.</param>
    /// <param name="open">Model expansion, or minus one for a leaf.</param>
    /// <param name="enabled">Whether the chevron accepts input.</param>
    /// <param name="track">Whether to expose inspection geometry.</param>
    /// <returns>Whether expansion was toggled.</returns>
    public bool TreeCell(string id, string label, string icon, int depth, ref int open, bool enabled, bool track)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        ArgumentOutOfRangeException.ThrowIfNegative(depth);
        bool changed = NativeMethods.TreeCell(context, Stable(id), label, Optional(icon), depth, ref open,
            enabled ? 1 : 0, out int part) != 0;
        if (track) TrackControl(id, part);
        return changed;
    }

    /// <summary>Begins the native virtual tree table.</summary>
    /// <param name="id">Stable tree table identity.</param>
    /// <param name="columns">Columns in display order.</param>
    /// <param name="labels">Literal node names for native type-ahead.</param>
    /// <param name="parents">Parent index per node, or minus one for roots.</param>
    /// <param name="flags">Native node flags.</param>
    /// <param name="state">Sort column, descending flag and cursor index.</param>
    /// <param name="heightKind">Encoded viewport extent kind.</param>
    /// <param name="height">Height or percentage.</param>
    /// <param name="rowHeight">Logical row height.</param>
    /// <param name="options">Input, guides and focus bits.</param>
    public unsafe void BeginTreeTable(string id, CuiGridColumn[] columns, string[] labels, int[] parents,
        uint[] flags, int[] state, int heightKind, float height, float rowHeight, int options)
    {
        CheckFrame();
        CheckGridHeight(labels.Length, heightKind, height, rowHeight);
        if (labels.Length != parents.Length || labels.Length != flags.Length || labels.Length >= int.MaxValue / 2)
            throw new ArgumentException("Expected matching node arrays.", nameof(parents));
        if (state.Length != 3 || state[0] < -2 || state[0] >= columns.Length ||
            state[2] < -1 || state[2] >= labels.Length)
            throw new ArgumentException("Invalid sort or cursor.", nameof(state));
        for (int index = 0; index < labels.Length; index++)
        {
            Utf8.Validate(labels[index], nameof(labels));
            if (parents[index] < -1 || parents[index] >= index)
                throw new ArgumentException("Parents must precede their children.", nameof(parents));
        }

        var (titles, widths, columnFlags) = GridColumns(columns);
        nint handle;
        int node;
        fixed (float* w = widths)
        fixed (int* c = columnFlags, p = parents, s = state)
        fixed (uint* f = flags)
            handle = NativeMethods.TreeTableBegin(context, Stable(id), titles, w, c, titles.Length,
                labels, p, f, labels.Length, s, heightKind, height, rowHeight, options, out node);
        layouts.Add(new(Layout.TreeTable, node, new TreeOutput(handle, flags, state)));
    }

    /// <summary>Begins the next native visible node.</summary>
    /// <param name="rowId">Optional inspection key by model index.</param>
    /// <returns>The node index, or minus one at the end.</returns>
    public int NextTreeTableRow(Func<int, string>? rowId)
    {
        RequireLayout(Layout.TreeTable);
        int index = NativeMethods.TreeTableRow(context, out int part);
        if (index >= 0 && rowId is not null) trackedNodes.Add(rowId(index), part);
        return index;
    }

    /// <summary>Draws the first cell of the current native row.</summary>
    /// <param name="label">Literal node name.</param>
    /// <param name="icon">Optional CUI glyph.</param>
    /// <param name="chevronId">Optional inspection key for a realized chevron.</param>
    /// <returns>Whether an expansion chevron was drawn.</returns>
    public bool TreeTableName(string label, string icon, string? chevronId)
    {
        RequireLayout(Layout.TreeTable);
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        int node = NativeMethods.TreeTableName(context, label, Optional(icon));
        if (node >= 0 && chevronId is not null) trackedNodes.Add(chevronId, node);
        return node >= 0;
    }

    /// <summary>Closes the tree table and copies its model changes.</summary>
    /// <returns>Native activation and lazy-load events.</returns>
    public unsafe CuiTreeTableEvents EndTreeTable()
    {
        var output = (TreeOutput)RequireLayout(Layout.TreeTable).Data!;
        int* events = stackalloc int[2];
        int* load = stackalloc int[64];
        fixed (uint* f = output.Flags)
        fixed (int* s = output.State)
            NativeMethods.TreeTableEnd(context, output.Handle, f, s, events, load);
        layouts.RemoveAt(layouts.Count - 1);
        return new(events[0], new ReadOnlySpan<int>(load, events[1]).ToArray());
    }
}
