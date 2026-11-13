using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A native ticker entry.</summary>
/// <param name="Label">Visible label.</param>
/// <param name="Value">Optional secondary value.</param>
/// <param name="Delta">Percent change, with zero hiding the trend.</param>
/// <param name="LowerIsBetter">Whether a fall is favorable.</param>
public readonly record struct CuiTickerItem(string Label, string Value, float Delta, bool LowerIsBetter);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native ticker strip.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="items">Ticker entries in display order.</param>
    /// <param name="speed">Logical points per second, or zero for the default.</param>
    /// <param name="height">Logical height, or zero for the default.</param>
    public unsafe void Ticker(string id, CuiTickerItem[] items, float speed, float height)
    {
        CheckFrame();
        CheckFeedbackNumber(speed, nameof(speed));
        CheckFeedbackNumber(height, nameof(height));
        using var memory = new ChartMemory();
        var data = new NativeTickerItem[items.Length];
        for (int i = 0; i < data.Length; i++)
        {
            var item = items[i];
            Utf8.Validate(item.Label, nameof(items));
            Utf8.Validate(item.Value, nameof(items));
            if (!float.IsFinite(item.Delta)) throw new ArgumentOutOfRangeException(nameof(items));
            data[i] = new() { Label = memory.Text(item.Label), Value = memory.Text(Optional(item.Value)),
                Delta = item.Delta, LowerIsBetter = item.LowerIsBetter ? 1 : 0 };
        }
        fixed (NativeTickerItem* native = data)
            NativeMethods.Ticker(context, Stable(id), native, data.Length, speed, height);
    }
}
