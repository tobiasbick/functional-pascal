using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native dot-matrix chart options.</summary>
/// <param name="Chart">Shared dimensions, ranges, formatting and annotations.</param>
/// <param name="Minimum">First X coordinate.</param>
/// <param name="Maximum">Last X coordinate.</param>
/// <param name="Step">Axis label spacing, or zero for automatic spacing.</param>
/// <param name="Units">Numeric labels or hours of the day.</param>
/// <param name="XFormat">Optional numeric axis format.</param>
/// <param name="Unit">Value unit.</param>
/// <param name="Pitch">Distance between dots, or zero for the default.</param>
/// <param name="Axis">Whether to show the axis labels.</param>
/// <param name="Readout">Whether to show the interactive value label.</param>
public sealed record CuiDotOptions(CuiChartOptions Chart, double Minimum, double Maximum, double Step,
    int Units, CuiChartFormat? XFormat, string Unit, float Pitch, bool Axis, bool Readout);

/// <summary>Native activation and readout.</summary>
/// <param name="Clicked">Whether the pointer or keyboard activated the chart.</param>
/// <param name="Index">Sample index, or minus one.</param>
/// <param name="X">X coordinate in axis units.</param>
/// <param name="Value">Stacked value at the sample.</param>
public readonly record struct CuiDotHit(bool Clicked, int Index, double X, float Value);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native dot chart and reports its current readout.</summary>
    /// <param name="id">Stable chart identity.</param>
    /// <param name="samples">Flattened series in band order.</param>
    /// <param name="accent">One accent flag per band.</param>
    /// <param name="count">Samples per band.</param>
    /// <param name="settings">Native chart and axis settings.</param>
    /// <param name="enabled">Whether the chart accepts input.</param>
    /// <param name="focus">Whether to focus the chart.</param>
    /// <returns>Activation, sample index and plotted values.</returns>
    public unsafe CuiDotHit DotChart(string id, float[] samples, int[] accent, int count,
        CuiDotOptions settings, bool enabled, bool focus)
    {
        CheckFrame();
        CheckExtent(settings.Chart.WidthKind, settings.Chart.Width, nameof(settings));
        CheckFeedbackNumber(settings.Chart.Height, nameof(settings));
        CheckFeedbackNumber(settings.Pitch, nameof(settings));
        Utf8.Validate(settings.Unit, nameof(settings));
        if (accent.Length > 16 || count < 0 || (long)accent.Length * count != samples.Length ||
            !double.IsFinite(settings.Minimum) || !double.IsFinite(settings.Maximum) ||
            !double.IsFinite(settings.Maximum - settings.Minimum) || !double.IsFinite(settings.Step) ||
            settings.Maximum < settings.Minimum || settings.Step < 0 || settings.Units is < 0 or > 1 ||
            settings.Units == 1 && (Math.Abs(settings.Minimum) > 35_000_000 ||
                Math.Abs(settings.Maximum) > 35_000_000 || settings.Minimum == settings.Maximum && count > 35_000_000))
            throw new ArgumentException("Invalid dot chart series or axis.", nameof(settings));
        using var memory = new ChartMemory();
        var native = new NativeDotOptions {
            Chart = memory.Options(settings.Chart), Minimum = settings.Minimum, Maximum = settings.Maximum,
            Step = settings.Step, Units = settings.Units, XFormat = memory.Format(settings.XFormat),
            Unit = memory.Text(Optional(settings.Unit)), Pitch = settings.Pitch,
            Flags = (settings.Axis ? 0 : 1) | (settings.Readout ? 0 : 2) | (enabled ? 4 : 0) | (focus ? 8 : 0)
        };
        fixed (float* values = samples)
        fixed (int* bands = accent)
        {
            int clicked = NativeMethods.DotChart(context, Stable(id), values, bands, accent.Length,
                count, in native, out int index, out double x, out float value);
            return new(clicked != 0, index, x, value);
        }
    }
}
