using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's patterned capacity bar and native interactive legend.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="parts">Up to 32 named nonnegative amounts.</param>
    /// <param name="options">Native capacity and presentation.</param>
    /// <param name="enabled">Allow native keyboard and pointer interaction.</param>
    /// <param name="focus">Request focus on the native bar.</param>
    /// <param name="track">Retain inspection and focus identity.</param>
    /// <returns>Native hovered or keyboard-selected part, otherwise minus one.</returns>
    public unsafe int PatternMeter(string id, CuiMeterPart[] parts, CuiMeterOptions options,
        bool enabled = true, bool focus = false, bool track = true)
    {
        CheckFrame(); string stable = Stable(id);
        ArgumentNullException.ThrowIfNull(parts); ArgumentNullException.ThrowIfNull(options);
        if (parts.Length > 32) throw new ArgumentOutOfRangeException(nameof(parts));
        float[] values = new float[parts.Length];
        for (int i = 0; i < parts.Length; i++)
        {
            ArgumentNullException.ThrowIfNull(parts[i]); Utf8.Validate(parts[i].Name, nameof(parts));
            CheckChartColor(parts[i].Color);
            if (!float.IsFinite(parts[i].Density) || parts[i].Density < 0 || parts[i].Density > 1)
                throw new ArgumentOutOfRangeException(nameof(parts));
            values[i] = parts[i].Value;
        }
        CheckChartValues(values); CheckExtent(options.WidthKind, options.Width, nameof(options));
        if (!float.IsFinite(options.Total) || options.Total < 0 || !float.IsFinite(options.Height) || options.Height < 0)
            throw new ArgumentOutOfRangeException(nameof(options));
        Utf8.Validate(options.Label, nameof(options)); Utf8.Validate(options.Unit, nameof(options));
        if (options.FreeLabel is not null) Utf8.Validate(options.FreeLabel, nameof(options));
        CheckChartFormat(options.Format);
        using var memory = new ChartMemory();
        var native = memory.Options(options);
        int shown = NativeMethods.PatternMeter(context, stable, memory.Parts(parts), parts.Length,
            in native, enabled ? 1 : 0, focus ? 1 : 0, out int node, out uint key);
        if (track) TrackControl(id, node, key);
        return shown;
    }
}
