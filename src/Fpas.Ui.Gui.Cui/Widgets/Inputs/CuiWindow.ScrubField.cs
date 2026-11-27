// [Documentation](../../../../docs/ui/gui/smart-inputs/scrub-field.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Charts;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's draggable numeric field and native typing mode.</summary>
    /// <param name="id">Stable identity independent of the caption.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="value">Finite model value within the bounds.</param>
    /// <param name="minimum">Finite lower bound.</param>
    /// <param name="maximum">Finite upper bound greater than the minimum.</param>
    /// <param name="step">Positive increment or zero for one percent of the range.</param>
    /// <param name="format">Number format. Null uses zero fractional digits.</param>
    /// <param name="suffix">Literal unit suffix. Null omits its position.</param>
    /// <param name="icon">Glyph replacing the caption. Null shows the caption and empty hides it.</param>
    /// <param name="widthKind">Content zero, fill one, fixed two or percentage three.</param>
    /// <param name="width">Logical extent or percentage.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to retain and inspect the control.</param>
    /// <returns>The proposed model value.</returns>
    public float ScrubField(string id, string label, float value, float minimum, float maximum,
        float step, CuiChartFormat? format, string? suffix, string? icon, int widthKind, float width,
        bool enabled, bool focus, bool track)
    {
        CheckFrame(); string identity = Stable(id);
        Utf8.Validate(label, nameof(label));
        if (!float.IsFinite(value) || !float.IsFinite(minimum) || !float.IsFinite(maximum) ||
            minimum >= maximum || value < minimum || value > maximum)
            throw new ArgumentException("A scrub needs a finite value within increasing finite bounds.");
        CheckChartFormat(format);
        if (suffix is not null) Utf8.Validate(suffix, nameof(suffix));
        if (icon is not null) Utf8.Validate(icon, nameof(icon));
        CheckSmartWidth(widthKind, width);
        float range = maximum - minimum;
        if (!float.IsFinite(range) || !float.IsFinite(step) || step < 0)
            throw new ArgumentOutOfRangeException(nameof(step));
        float increment = step > 0 ? step : range / 100;
        float fineStep = increment > 0 ? increment / 10 : .1f;
        // CUI-039: native dragging divides the current value by the fine step before rounding.
        if (fineStep <= 0 || !float.IsFinite(MathF.Max(MathF.Abs(minimum), MathF.Abs(maximum)) / fineStep))
            throw new ArgumentOutOfRangeException(nameof(step), "The scrub step is too small for fine dragging within its bounds.");
        NativeMethods.ScrubField(context, identity, label, ref value, minimum, maximum, step,
            ChartMemory.FormatString(format), suffix, icon, widthKind, width,
            enabled ? 1 : 0, focus ? 1 : 0, out int node, out uint key);
        if (track) TrackControl(id, node, key);
        return value;
    }
}
