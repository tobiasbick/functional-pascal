// [Documentation](../../../../docs/ui/gui/canvas-and-bars/selection-bar.md)
using Fpas.Ui.Gui.Cui.Interop.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a floating bar for bulk actions beneath a widget.</summary>
    /// <param name="id">Stable identity for the selection bar.</param>
    /// <param name="selected">Item flags. The caller applies a clear proposal.</param>
    /// <param name="actions">Actions in display order.</param>
    /// <param name="enabled">Whether the bar accepts input.</param>
    /// <param name="noEscape">Whether Escape leaves the selection intact.</param>
    /// <param name="count">Aggregate nonnegative count when selected is null.</param>
    /// <param name="countFormat">At most one %d slot, %% for a percent; empty uses CUI's default.</param>
    /// <param name="anchor">The native anchor ID in the current scope, or null for the previous widget.</param>
    /// <returns>The zero-based action index, -2 for a clear proposal, or -1 when idle.</returns>
    public unsafe int SelectionBar(string id, bool[]? selected, CuiSelectionAction[] actions, bool enabled, bool noEscape,
        int count = 0, string countFormat = "", string? anchor = null)
    {
        CheckFrame();
        ArgumentOutOfRangeException.ThrowIfNegative(count);
        if (selected is not null && count != 0) throw new ArgumentException("Supply item flags or a count.", nameof(count));
        string? format = CountFormat.Check(countFormat, 1);
        using var memory = new ChartMemory();
        var native = Array.ConvertAll(actions, action => new NativeSelectionAction
        {
            Icon = string.IsNullOrEmpty(action.Icon) ? null : memory.Text(action.Icon),
            Label = memory.Text(action.Label),
            Tone = action.Tone
        });
        byte[] values = selected is null ? [] : Array.ConvertAll(selected, value => value ? (byte)1 : (byte)0);
        fixed (byte* rows = values)
            return NativeMethods.SelectionBar(context, Stable(id), rows, values.Length,
                count, memory.Copy<NativeSelectionAction>(native), actions.Length, enabled ? 1 : 0, noEscape ? 1 : 0,
                format, anchor is null ? null : Stable(anchor));
    }
}
