using Fpas.Ui.Gui.Cui.Interop.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native SelectionBar operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="selected">selected supplied to the native operation.</param>
    /// <param name="actions">actions supplied to the native operation.</param>
    /// <param name="enabled">enabled supplied to the native operation.</param>
    /// <param name="noEscape">noEscape supplied to the native operation.</param>
    /// <param name="count">Aggregate nonnegative count when selected is null.</param>
    /// <param name="countFormat">At most one %d slot, %% for a percent; empty uses CUI's default.</param>
    /// <returns>The native result.</returns>
    public unsafe int SelectionBar(string id, bool[]? selected, CuiSelectionAction[] actions, bool enabled, bool noEscape,
        int count = 0, string countFormat = "")
    {
        CheckFrame();
        ArgumentOutOfRangeException.ThrowIfNegative(count);
        if (selected is not null && count != 0) throw new ArgumentException("Supply item flags or a count.", nameof(count));
        string? format = CountFormat.Check(countFormat, 1);
        using var memory = new ChartMemory();
        var native = Array.ConvertAll(actions, action => new NativeSelectionAction
        {
            Icon = memory.Text(action.Icon), Label = memory.Text(action.Label), Tone = action.Tone
        });
        byte[] values = selected is null ? [] : Array.ConvertAll(selected, value => value ? (byte)1 : (byte)0);
        fixed (byte* rows = values)
            return NativeMethods.SelectionBar(context, Stable(id), rows, values.Length,
                count, memory.Copy<NativeSelectionAction>(native), actions.Length, enabled ? 1 : 0, noEscape ? 1 : 0, format);
    }
}
