using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native data for SelectionBar.</summary>
/// <param name="Icon">Icon supplied to CUI.</param>
/// <param name="Label">Label supplied to CUI.</param>
/// <param name="Tone">Tone supplied to CUI.</param>
public sealed record CuiSelectionAction(string Icon, string Label, int Tone);

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native SelectionBar operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="selected">selected supplied to the native operation.</param>
    /// <param name="actions">actions supplied to the native operation.</param>
    /// <param name="enabled">enabled supplied to the native operation.</param>
    /// <param name="noEscape">noEscape supplied to the native operation.</param>
    /// <returns>The native result.</returns>
    public unsafe int SelectionBar(string id, bool[] selected, CuiSelectionAction[] actions, bool enabled, bool noEscape)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        var native = Array.ConvertAll(actions, action => new NativeSelectionAction
        {
            Icon = memory.Text(action.Icon), Label = memory.Text(action.Label), Tone = action.Tone
        });
        byte[] values = Array.ConvertAll(selected, value => value ? (byte)1 : (byte)0);
        fixed (byte* rows = values)
            return NativeMethods.SelectionBar(context, Stable(id), rows, values.Length,
                memory.Copy<NativeSelectionAction>(native), actions.Length, enabled ? 1 : 0, noEscape ? 1 : 0);
    }
}
