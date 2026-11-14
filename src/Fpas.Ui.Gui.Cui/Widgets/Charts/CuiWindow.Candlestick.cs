using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's retained candlestick/OHLC viewport, crosshair and volume.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="candles">Valid finite OHLC periods and nonnegative volume.</param>
    /// <param name="options">Native labels, format, overlays and colors.</param>
    /// <param name="enabled">Allow native pointer, wheel and keyboard input.</param>
    /// <param name="focus">Request native keyboard focus.</param>
    /// <param name="track">Track inspection and retirement identity.</param>
    /// <returns>Native crosshair input index or minus one.</returns>
    public unsafe int Candlestick(string id, CuiCandle[] candles, CuiCandlestickOptions options,
        bool enabled = true, bool focus = false, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckCandles(candles, options);
        using var memory = new ChartMemory();
        var native = memory.Options(options);
        int cursor = NativeMethods.Candlestick(context, stable, memory.Copy<CuiCandle>(candles), candles.Length,
            in native, enabled ? 1 : 0, focus ? 1 : 0, out int node);
        if (track) TrackControl(id, node); return cursor;
    }
}
