using Fpas.Ui.Gui.Cui.Widgets.Collections;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private sealed class GridOutput(nint handle, CuiGridFrame frame)
    {
        internal nint Handle { get; } = handle;
        internal CuiGridFrame Frame { get; } = frame;
        internal int NextRow { get; set; } = frame.First;
    }

    /// <summary>Begins the native grid with frame-local selection and sort data.</summary>
    /// <param name="id">Stable grid identity.</param>
    /// <param name="columns">Native column descriptors.</param>
    /// <param name="rows">Total number of rows.</param>
    /// <param name="selected">Input selection, or null.</param>
    /// <param name="state">Sort column, descending flag, cursor and anchor.</param>
    /// <param name="heightKind">Encoded viewport extent kind.</param>
    /// <param name="height">Height or percentage.</param>
    /// <param name="rowHeight">Logical row height.</param>
    /// <param name="options">Native input, striping, checkbox and focus bits.</param>
    /// <param name="headerId">Optional inspection key by column index.</param>
    /// <returns>Frame-local visible range and output buffers.</returns>
    public unsafe CuiGridFrame BeginDataGrid(string id, CuiGridColumn[] columns, int rows, bool[]? selected,
        int[] state, int heightKind, float height, float rowHeight, int options, Func<int, string>? headerId)
    {
        CheckFrame();
        CheckGridHeight(rows, heightKind, height, rowHeight);
        if (selected is not null && selected.Length != rows)
            throw new ArgumentException("Expected one selection value per row.", nameof(selected));
        if (state.Length != 4 || state[0] < -2 || state[0] >= columns.Length ||
            state[2] < -1 || state[2] >= rows || state[3] < -1 || state[3] >= rows)
            throw new ArgumentException("Invalid grid sort, cursor or anchor.", nameof(state));

        var (titles, widths, flags) = GridColumns(columns);
        var selection = selected is null ? null : new byte[Math.Max(1, rows)];
        if (selected is not null)
            for (int index = 0; index < rows; index++) selection![index] = selected[index] ? (byte)1 : (byte)0;

        int* range = stackalloc int[3];
        var headers = new int[columns.Length];
        nint handle;
        fixed (float* w = widths)
        fixed (int* f = flags, s = state, h = headers)
        fixed (byte* mask = selection)
            handle = NativeMethods.DataGridBegin(context, Stable(id), titles, w, f, titles.Length,
                rows, mask, s, heightKind, height, rowHeight, options, range, h);
        var shown = new bool[columns.Length];
        if (headerId is not null)
            for (int index = 0; index < headers.Length; index++)
                if (shown[index] = headers[index] >= 0)
                    TrackControl(headerId(index), headers[index]);
        var frame = new CuiGridFrame(range[0], range[1], selection, state, shown);
        layouts.Add(new(Layout.DataGrid, range[2], new GridOutput(handle, frame)));
        return frame;
    }

    /// <summary>Begins a realized row in display order.</summary>
    /// <param name="position">Row's current display index.</param>
    /// <param name="id">Stable row identity.</param>
    /// <param name="track">Whether to expose row geometry.</param>
    public void DataGridRow(int position, string id, bool track)
    {
        var output = (GridOutput)RequireLayout(Layout.DataGrid).Data!;
        if (position != output.NextRow || position >= output.Frame.Last)
            throw new ArgumentOutOfRangeException(nameof(position), "Build the realized rows in order.");
        int node = NativeMethods.DataGridRow(context, position, Stable(id));
        output.NextRow++;
        if (track) trackedNodes.Add(id, node);
    }

    /// <summary>Advances to the next native grid cell.</summary>
    /// <param name="text">Literal native cell text, or null for composed content.</param>
    public void DataGridCell(string? text)
    {
        CheckFrame();
        if (layouts.Count == 0 || layouts[^1].Kind is not (Layout.DataGrid or Layout.TreeTable))
            throw new InvalidOperationException("Expected a native grid.");
        if (text is not null) Utf8.Validate(text, nameof(text));
        NativeMethods.DataGridCell(context, text);
    }

    /// <summary>Draws native editable text in the current cell.</summary>
    /// <param name="value">Current model value.</param>
    /// <param name="capacity">UTF-8 capacity including the terminator, at most 256.</param>
    /// <returns>Committed text, or null while idle, editing or canceled.</returns>
    public unsafe string? DataGridEdit(string value, int capacity)
    {
        RequireLayout(Layout.DataGrid);
        byte[] buffer = GridTextBuffer(value, capacity, 256);
        fixed (byte* text = buffer)
            return NativeMethods.DataGridEdit(context, text, capacity) != 0 ? ReadGridText(buffer) : null;
    }

    /// <summary>Closes a native grid and collects the proposed state.</summary>
    public unsafe void EndDataGrid()
    {
        var output = (GridOutput)RequireLayout(Layout.DataGrid).Data!;
        fixed (byte* selected = output.Frame.Selected)
        fixed (int* state = output.Frame.State)
            NativeMethods.DataGridEnd(context, output.Handle, selected, state);
        layouts.RemoveAt(layouts.Count - 1);
    }

    private static (string[] Titles, float[] Widths, int[] Flags) GridColumns(CuiGridColumn[] columns)
    {
        if (columns.Length is < 1 or > 32) throw new ArgumentOutOfRangeException(nameof(columns));
        var titles = new string[columns.Length];
        var widths = new float[columns.Length];
        var flags = new int[columns.Length];
        for (int index = 0; index < columns.Length; index++)
        {
            var column = columns[index];
            Utf8.Validate(column.Title, nameof(columns));
            if (!float.IsFinite(column.Width) || column.Width < 0 || column.Alignment is < 0 or > 2 ||
                column.Priority is < 0 or > 255)
                throw new ArgumentException("Invalid width, alignment or priority.", nameof(columns));
            titles[index] = column.Title;
            widths[index] = column.Width;
            flags[index] = column.Alignment | (column.Sortable ? 8 : 0) |
                (column.Fixed ? 16 : 0) | (column.Priority << 5);
        }
        return (titles, widths, flags);
    }

    private static void CheckGridHeight(int count, int kind, float height, float rowHeight)
    {
        CheckExtent(kind, height, nameof(height));
        if (count < 0 || !float.IsFinite(rowHeight) || rowHeight < 1 || (count + 1d) * rowHeight >= int.MaxValue)
            throw new ArgumentOutOfRangeException(nameof(rowHeight));
    }

    private static byte[] GridTextBuffer(string value, int capacity, int maximum)
    {
        Utf8.Validate(value, nameof(value));
        if (capacity < 2 || capacity > maximum || Utf8.Encoding.GetByteCount(value) >= capacity)
            throw new ArgumentException("Text does not fit its native UTF-8 buffer.", nameof(capacity));
        var buffer = new byte[capacity];
        Utf8.Encoding.GetBytes(value.AsSpan(), buffer);
        return buffer;
    }

    private static string ReadGridText(byte[] buffer) =>
        Utf8.Encoding.GetString(buffer.AsSpan(0, Array.IndexOf(buffer, (byte)0)));
}
