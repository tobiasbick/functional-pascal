using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI gradient slider.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption.</param>
    /// <param name="value">Model value and native proposal.</param>
    /// <param name="minimum">Lower endpoint.</param>
    /// <param name="maximum">Upper endpoint.</param>
    /// <param name="track">Native gradient preset.</param>
    /// <param name="positions">Normalized gradient positions.</param>
    /// <param name="colors">Packed RGBA colors.</param>
    /// <param name="color">Semantic opacity track color.</param>
    /// <param name="step">Logical or normalized step.</param>
    /// <param name="format">Optional native numeric format.</param>
    /// <param name="display">Optional display caption.</param>
    /// <param name="tool">Optional tool glyph.</param>
    /// <param name="tooltip">Optional tool tooltip.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <param name="sample">The packed color under the thumb.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int GradientSlider(string id, string label, ref float value, float minimum, float maximum, int track, float[] positions, uint[] colors, CuiChartColor color, float step, CuiChartFormat? format, string display, string tool, string tooltip,
        int flags, int focus, Func<int, string>? part, out uint sample)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(display, nameof(display));
        Utf8.Validate(tool, nameof(tool));
        Utf8.Validate(tooltip, nameof(tooltip));
        if (!float.IsFinite(value) || !float.IsFinite(minimum) || !float.IsFinite(maximum - minimum) ||
            maximum <= minimum || value < minimum || value > maximum || track is < 0 or > 2)
            throw new ArgumentOutOfRangeException(nameof(value));
        if (positions.Length != colors.Length) throw new ArgumentException("Expected aligned gradient stops.");
        float previous = -1;
        foreach (float position in positions)
        {
            if (!float.IsFinite(position) || position is < 0 or > 1 || position < previous)
                throw new ArgumentException("Gradient stops must be sorted normalized positions.", nameof(positions));
            previous = position;
        }
        CheckFeedbackNumber(step, nameof(step));
        CheckChartFormat(format);
        CheckChartColor(color);
        using var memory = new ChartMemory();
        int count = 2;
        int* nodes = stackalloc int[count];
        int result;
        fixed (float* p = positions)
        fixed (uint* c = colors)
            result = NativeMethods.GradientSlider(context, Stable(id), label, ref value, minimum, maximum, track, p, c, positions.Length, color.Palette, color.Rgba, step, memory.Format(format), Optional(display), Optional(tool), Optional(tooltip), flags, focus, out sample, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
