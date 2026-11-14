using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Planning;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a native Kanban board.</summary>
    /// <param name="id">Stable board identity.</param>
    /// <param name="enabled">Whether the board accepts input.</param>
    public void BeginKanban(string id, bool enabled)
    {
        CheckFrame();
        int node = NativeMethods.KanbanBegin(context, Stable(id), enabled ? 1 : 0);
        layouts.Add(new(Layout.Kanban, Node: node));
    }

    /// <summary>Opens the next native board column.</summary>
    /// <param name="title">Column title.</param>
    /// <param name="count">Number of cards in this column.</param>
    /// <param name="dot">Optional heading color.</param>
    /// <param name="limit">Work-in-progress limit, or zero.</param>
    /// <param name="width">Logical width, or zero.</param>
    /// <param name="columnToken">Optional tracking identity of the column.</param>
    /// <param name="addToken">Optional tracking identity of Add.</param>
    /// <returns>Whether Add was activated.</returns>
    public unsafe bool BeginKanbanColumn(string title, int count, CuiChartColor dot, int limit, float width,
        string? columnToken, string? addToken)
    {
        RequireLayout(Layout.Kanban);
        Utf8.Validate(title, nameof(title));
        CheckChartColor(dot);
        CheckFeedbackNumber(width, nameof(width));
        if (count < 0 || limit < 0) throw new ArgumentOutOfRangeException(nameof(count));
        int* parts = stackalloc int[2];
        int result = NativeMethods.KanbanColumnBegin(context, title, count, dot.Palette, dot.Rgba, limit, width, parts);
        if (columnToken is not null) TrackControl(columnToken, parts[0]);
        if (addToken is not null) TrackControl(addToken, parts[1]);
        layouts.Add(new(Layout.KanbanColumn));
        return result != 0;
    }

    /// <summary>Opens a native board card for application-provided contents.</summary>
    /// <param name="key">Stable card identity.</param>
    /// <param name="focus">Whether to focus the card.</param>
    /// <param name="token">Optional tracking identity.</param>
    /// <returns>Whether the card was activated.</returns>
    public bool BeginKanbanCard(int key, bool focus, string? token)
    {
        RequireLayout(Layout.KanbanColumn);
        if (key < 0) throw new ArgumentOutOfRangeException(nameof(key));
        int result = NativeMethods.KanbanCardBegin(context, key, focus ? 1 : 0, out int node);
        if (token is not null) TrackControl(token, node);
        layouts.Add(new(Layout.KanbanCard));
        return result != 0;
    }

    /// <summary>Closes the card's content.</summary>
    public void EndKanbanCard()
    {
        RequireLayout(Layout.KanbanCard);
        NativeMethods.KanbanCardEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Closes the current column.</summary>
    public void EndKanbanColumn()
    {
        RequireLayout(Layout.KanbanColumn);
        NativeMethods.KanbanColumnEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Closes the board and reports a completed drop.</summary>
    /// <returns>A native move or null.</returns>
    public CuiKanbanMove? EndKanban()
    {
        var scope = RequireLayout(Layout.Kanban);
        int result = NativeMethods.KanbanEnd(context, scope.Node, out var move);
        layouts.RemoveAt(layouts.Count - 1);
        return result != 0 ? move : null;
    }
}
