// [Documentation](../../../../docs/ui/gui/canvas-and-bars/result-card.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Interop.Media;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws an activatable image card with a window-owned native image cache.</summary>
    /// <param name="id">Stable card identity.</param>
    /// <param name="image">Immutable RGBA pixels, an image filename, or null for a muted surface.</param>
    /// <param name="title">Visible card title.</param>
    /// <param name="caption">Optional secondary caption.</param>
    /// <param name="badge">Optional badge caption.</param>
    /// <param name="tone">Native semantic tone ordinal for the badge. Zero uses success.</param>
    /// <param name="widthKind">Fill capped at 320=0, fill=1, logical points=2, percentage=3.</param>
    /// <param name="width">Nonnegative logical width or percentage, according to the extent kind.</param>
    /// <param name="aspect">Card height divided by width. Zero uses 1.25.</param>
    /// <param name="enabled">Whether the card accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="stretchImage">Whether the picture stretches to the card.</param>
    /// <returns>Whether the enabled card was activated.</returns>
    public unsafe bool ResultCard(string id, CuiMediaImage? image, string title, string? caption, string? badge,
        int tone, int widthKind, float width, float aspect, bool enabled, bool focus, bool stretchImage = false)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        if (caption is not null) Utf8.Validate(caption, nameof(caption));
        if (badge is not null) Utf8.Validate(badge, nameof(badge));
        CheckTone(tone);
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(aspect, nameof(aspect));
        using var memory = new ChartMemory();
        fixed (byte* pixels = image?.Pixels)
        {
            NativeMediaSource source = image is { } picture ? MediaSource(picture, memory, pixels) : default;
            return NativeMethods.ResultCard(context, Stable(id), image.HasValue ? &source : null, title, caption,
                badge, tone, widthKind, width, aspect, enabled ? 1 : 0, focus ? 1 : 0, stretchImage ? 1 : 0) != 0;
        }
    }
}
