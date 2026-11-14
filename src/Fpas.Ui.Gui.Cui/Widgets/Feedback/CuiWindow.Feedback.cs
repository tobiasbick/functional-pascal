using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a CUI-owned inline alert and tracks its optional dismiss action.</summary>
    /// <param name="id">Stable alert identity, retained while a dismissed alert remains in the tree.</param>
    /// <param name="title">Literal heading.</param>
    /// <param name="description">Optional supporting text.</param>
    /// <param name="icon">Optional icon glyph; empty uses the tone icon.</param>
    /// <param name="tone">Native tone from zero through eight.</param>
    /// <param name="dismissible">Whether to provide the native dismiss action.</param>
    /// <param name="enabled">Whether that action accepts input.</param>
    /// <param name="focus">Whether to request action focus.</param>
    /// <param name="dismissId">Inspection identity of the dismiss action.</param>
    /// <returns>Visible bit 1 and newly-dismissed bit 2.</returns>
    public int Alert(string id, string title, string description, string icon, int tone,
        bool dismissible, bool enabled, bool focus, string dismissId)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(description, nameof(description));
        Utf8.Validate(icon, nameof(icon));
        if (string.IsNullOrWhiteSpace(title)) throw new ArgumentException("An alert needs a title.", nameof(title));
        ArgumentException.ThrowIfNullOrEmpty(dismissId);
        CheckTone(tone);
        int result = NativeMethods.Alert(context, Stable(id), title, description, icon, tone,
            dismissible ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, out int node, out int dismiss, out uint key);
        TrackControl(id, node, key);
        if (dismiss >= 0) TrackControl(dismissId, dismiss);
        return result;
    }

    /// <summary>Draws native progress with CUI interpolation and activity animation.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="value">Finite fraction from zero through one.</param>
    /// <param name="tone">Native tone from zero through eight.</param>
    /// <param name="indeterminate">Whether to show activity instead of a fraction.</param>
    /// <param name="height">Logical track height; zero uses CUI's default.</param>
    /// <param name="label">Optional literal caption.</param>
    /// <param name="showValue">Whether to display the rounded percentage.</param>
    /// <param name="dither">Whether to draw the native ordered-dot fill.</param>
    /// <param name="widthKind">Encoded native width extent kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    public void Progress(string id, float value, int tone = 0, bool indeterminate = false,
        float height = 0, string label = "", bool showValue = false, bool dither = false,
        int widthKind = 0, float width = 0)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        CheckProgressValue(value);
        CheckTone(tone);
        CheckFeedbackNumber(height, nameof(height));
        CheckExtent(widthKind, width, nameof(width));
        NativeMethods.Progress(context, Stable(id), value, tone, indeterminate ? 1 : 0, height, label,
            showValue ? 1 : 0, dither ? 1 : 0, widthKind, width);
    }

    /// <summary>Draws CUI's native activity spinner.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="size">Logical diameter; zero uses the native default.</param>
    /// <param name="style">Native animation style.</param>
    /// <param name="color">Optional ink color.</param>
    /// <param name="track">Optional track color.</param>
    /// <param name="inverse">Use the current theme background as ink.</param>
    public void Spinner(string id, float size = 0, int style = 0, CuiChartColor color = default, CuiChartColor track = default, bool inverse = false)
    {
        CheckFrame();
        CheckFeedbackNumber(size, nameof(size));
        if (style is < 0 or > 14) throw new ArgumentOutOfRangeException(nameof(style));
        CheckChartColor(color);
        CheckChartColor(track);
        NativeMethods.Spinner(context, Stable(id), size, style, inverse ? -2 : color.Palette, color.Rgba, track.Palette, track.Rgba);
    }

    /// <summary>Draws CUI's usage meter with native formatting and threshold colors.</summary>
    /// <param name="id">Stable meter identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="used">Finite nonnegative usage; exceeding the capacity is allowed.</param>
    /// <param name="total">Finite nonnegative capacity; zero displays an empty track.</param>
    /// <param name="unit">Optional literal value suffix.</param>
    public void Meter(string id, string label, float used, float total, string unit = "")
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(unit, nameof(unit));
        CheckFeedbackNumber(used, nameof(used));
        CheckFeedbackNumber(total, nameof(total));
        NativeMethods.Meter(context, Stable(id), label, used, total, unit);
    }

    /// <summary>Draws CUI's circular progress, percentage and optional caption.</summary>
    /// <param name="id">Stable identity, independent of the caption.</param>
    /// <param name="label">Optional literal caption inside the ring.</param>
    /// <param name="value">Finite fraction from zero through one.</param>
    /// <param name="size">Logical diameter; zero uses CUI's 96-point default.</param>
    public void ProgressRing(string id, string label, float value, float size = 0)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        CheckProgressValue(value);
        CheckFeedbackNumber(size, nameof(size));
        NativeMethods.ProgressRing(context, Stable(id), label, value, size);
    }

    /// <summary>Draws CUI's native activity entry and optional connector.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="title">Nonempty literal heading.</param>
    /// <param name="time">Optional literal time caption.</param>
    /// <param name="description">Optional literal supporting text.</param>
    /// <param name="icon">Optional icon glyph; empty selects CUI's dot.</param>
    /// <param name="tone">Native semantic tone from zero through eight.</param>
    /// <param name="last">Whether to omit the connector and trailing space.</param>
    public void TimelineItem(string id, string title, string time = "", string description = "",
        string icon = "", int tone = 0, bool last = false)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(time, nameof(time));
        Utf8.Validate(description, nameof(description));
        Utf8.Validate(icon, nameof(icon));
        if (string.IsNullOrWhiteSpace(title)) throw new ArgumentException("A timeline item needs a title.", nameof(title));
        CheckTone(tone);
        NativeMethods.TimelineItem(context, Stable(id), title, time, description, icon, tone, last ? 1 : 0);
    }

    /// <summary>Draws CUI's native shimmer placeholder with an explicit width mode.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="widthKind">Fill one, logical pixels two, or percentage three.</param>
    /// <param name="width">Logical pixels or a percentage from zero through 100.</param>
    /// <param name="height">Logical height; zero uses CUI's 14-point default.</param>
    public void Skeleton(string id, int widthKind = 1, float width = 0, float height = 0)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        if (widthKind == 0) throw new ArgumentOutOfRangeException(nameof(widthKind), "A skeleton has no content width; use fill.");
        CheckFeedbackNumber(height, nameof(height));
        NativeMethods.Skeleton(context, Stable(id), widthKind, width, height);
    }

    private static void CheckProgressValue(float value)
    {
        if (!float.IsFinite(value) || value is < 0 or > 1) throw new ArgumentOutOfRangeException(nameof(value));
    }

    private static void CheckTone(int tone)
    {
        if (tone is < 0 or > 8) throw new ArgumentOutOfRangeException(nameof(tone));
    }

    private static void CheckFeedbackNumber(float value, string name)
    {
        if (!float.IsFinite(value) || value < 0) throw new ArgumentOutOfRangeException(name);
    }
}
