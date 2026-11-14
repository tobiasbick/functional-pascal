using Fpas.Ui.Gui.Cui.Interop.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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
