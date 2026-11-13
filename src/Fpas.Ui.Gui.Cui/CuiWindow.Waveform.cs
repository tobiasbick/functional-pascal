using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native waveform bars and optional seeking.</summary>
    /// <param name="id">Stable waveform identity.</param>
    /// <param name="peaks">Normalized peak values.</param>
    /// <param name="progress">Current position, or null for passive bars.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="height">Logical height or zero.</param>
    /// <param name="bar">Bar width or zero.</param>
    /// <param name="gap">Bar gap or zero.</param>
    /// <param name="color">Played bar color.</param>
    /// <param name="flags">Enabled and focus bits.</param>
    /// <returns>The proposed position, or null without progress.</returns>
    public unsafe float? Waveform(string id, float[] peaks, float? progress, int widthKind, float width,
        float height, float bar, float gap, CuiChartColor color, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(height, nameof(height));
        CheckFeedbackNumber(bar, nameof(bar));
        CheckFeedbackNumber(gap, nameof(gap));
        float position = progress.GetValueOrDefault();
        fixed (float* values = peaks)
            NativeMethods.Waveform(context, Stable(id), values, peaks.Length, progress.HasValue ? &position : null,
                widthKind, width, height, bar, gap, color.Palette, color.Rgba, flags);
        return progress.HasValue ? position : null;
    }
}
