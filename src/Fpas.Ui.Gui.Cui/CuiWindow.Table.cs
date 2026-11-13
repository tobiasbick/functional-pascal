using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A native column descriptor.</summary>
/// <param name="Title">Literal heading.</param>
/// <param name="Kind">Encoded extent kind.</param>
/// <param name="Width">Extent value or grow weight.</param>
/// <param name="Align">Native text alignment.</param>
/// <param name="Sortable">Whether the header can be sorted.</param>
public readonly record struct CuiTableColumn(string Title, int Kind, float Width, int Align, bool Sortable);

/// <summary>The sort proposed by the native header.</summary>
/// <param name="Column">Zero-based column, minus one for unsorted, or minus two for no sorting.</param>
/// <param name="Descending">Whether the order is descending.</param>
public readonly record struct CuiTableSort(int Column, bool Descending);

public sealed partial class CuiWindow
{
    /// <summary>Opens the native table and returns the sort proposed by its headers.</summary>
    /// <param name="id">Stable table identity.</param>
    /// <param name="columns">Column headings, alignment and sizing.</param>
    /// <param name="heightKind">Encoded viewport extent kind.</param>
    /// <param name="height">Logical viewport extent or percentage.</param>
    /// <param name="striped">Whether alternate rows are tinted.</param>
    /// <param name="compact">Whether to use smaller rows.</param>
    /// <param name="selectable">Whether rows can be activated.</param>
    /// <param name="enabled">Whether headings accept input.</param>
    /// <param name="sort">Current zero-based column, minus one for unsorted or minus two to disable sorting.</param>
    /// <param name="descending">Whether the current order is descending.</param>
    /// <param name="focusHeader">Zero-based heading to focus, or minus one.</param>
    /// <param name="trackHeader">Optional mapping from heading index to inspection identity.</param>
    /// <returns>The order proposed by CUI.</returns>
    public unsafe CuiTableSort BeginTable(string id, CuiTableColumn[] columns, int heightKind, float height,
        bool striped, bool compact, bool selectable, bool enabled, int sort, bool descending,
        int focusHeader, Func<int, string>? trackHeader)
    {
        CheckFrame();
        CheckExtent(heightKind, height, nameof(height));
        ArgumentNullException.ThrowIfNull(columns);
        int count = columns.Length;
        if (count == 0 || sort < -2 || sort >= count) throw new ArgumentOutOfRangeException(nameof(sort));
        var titles = new string[count];
        var kinds = new int[count];
        var widths = new float[count];
        var alignments = new int[count];
        var sortable = new int[count];
        var headers = trackHeader is null ? [] : new int[count];
        for (int i = 0; i < count; i++)
        {
            var col = columns[i];
            Utf8.Validate(col.Title, nameof(columns));
            CheckExtent(col.Kind, col.Width, nameof(columns));
            if (col.Align is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(columns));
            titles[i] = col.Title;
            kinds[i] = col.Kind;
            widths[i] = col.Width;
            alignments[i] = col.Align;
            sortable[i] = col.Sortable ? 1 : 0;
        }

        int desc = descending ? 1 : 0;
        int node;
        fixed (int* k = kinds, a = alignments, s = sortable, h = headers)
        fixed (float* w = widths)
            node = NativeMethods.TableBegin(context, Stable(id), titles, k, w, a, s, count, heightKind, height,
                striped ? 1 : 0, compact ? 1 : 0, selectable ? 1 : 0, enabled ? 1 : 0,
                ref sort, ref desc, focusHeader, h);
        layouts.Add(new(Layout.Table, Node: node));
        if (trackHeader is not null)
            for (int i = 0; i < count; i++) TrackControl(trackHeader(i), headers[i]);
        return new(sort, desc != 0);
    }

    /// <summary>Opens the next native row and reports activation.</summary>
    /// <param name="id">Stable row identity.</param>
    /// <param name="selected">Whether to show selection.</param>
    /// <param name="enabled">Whether this row accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="track">Whether to expose row geometry and focus for inspection.</param>
    /// <returns>Whether the row was activated.</returns>
    public bool BeginTableRow(string id, bool selected, bool enabled, bool focus, bool track)
    {
        RequireLayout(Layout.Table);
        int clicked = NativeMethods.TableRow(context, Stable(id), selected ? 1 : 0,
            enabled ? 1 : 0, focus ? 1 : 0, out int node);
        layouts.Add(new(Layout.TableRow));
        if (track) trackedNodes.Add(id, node);
        return clicked != 0;
    }

    /// <summary>Opens a cell for composed children or draws native plain text in it.</summary>
    /// <param name="text">Literal cell text, or null to draw children.</param>
    public void TableCell(string? text)
    {
        RequireLayout(Layout.TableRow);
        if (text is not null) Utf8.Validate(text, nameof(text));
        NativeMethods.TableCell(context, text);
    }

    /// <summary>Ends the row identity scope.</summary>
    public void EndTableRow()
    {
        RequireLayout(Layout.TableRow);
        NativeMethods.TableRowEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Closes the native table.</summary>
    public void EndTable()
    {
        var scope = RequireLayout(Layout.Table);
        NativeMethods.TableEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
