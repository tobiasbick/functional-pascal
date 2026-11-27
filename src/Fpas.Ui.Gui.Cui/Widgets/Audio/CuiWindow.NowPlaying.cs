// [Documentation](../../../../docs/ui/gui/motion/now-playing.md)
using Fpas.Ui.Gui.Cui.Interop.Media;
using Fpas.Ui.Gui.Cui.Widgets.Audio;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native transport, artwork and scrubber.</summary>
    /// <param name="id">Stable card identity.</param>
    /// <param name="title">Track title.</param>
    /// <param name="artist">Track artist.</param>
    /// <param name="artwork">Optional image file or RGBA pixels.</param>
    /// <param name="color">Artwork tile background.</param>
    /// <param name="position">Current position and output seek.</param>
    /// <param name="duration">Track duration in seconds.</param>
    /// <param name="playing">Current and proposed playback flag.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="flags">Compact and enabled bits.</param>
    /// <param name="focus">Part to focus, or minus one.</param>
    /// <param name="part">Optional snapshot identity by part index.</param>
    /// <returns>The native event flags and available parts.</returns>
    public unsafe CuiPlayerResult NowPlaying(string id, string title, string artist, CuiMediaImage? artwork, CuiChartColor color,
        ref float position, float duration, ref int playing, int widthKind, float width, int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(artist, nameof(artist));
        CheckExtent(widthKind, width, nameof(width));
        CheckChartColor(color);
        CheckFeedbackNumber(duration, nameof(duration));
        CheckFeedbackNumber(position, nameof(position));
        if (position > duration) throw new ArgumentOutOfRangeException(nameof(position));
        int* nodes = stackalloc int[4];
        using var memory = new ChartMemory();
        int change;
        fixed (byte* pixels = artwork?.Pixels)
        {
            NativeMediaSource source = artwork is { } image ? MediaSource(image, memory, pixels) : default;
            change = NativeMethods.NowPlaying(context, Stable(id), title, artist, artwork.HasValue ? &source : null,
                color.Palette, color.Rgba, ref position, duration, ref playing, widthKind, width, flags, focus, nodes);
        }
        var available = new bool[4];
        for (int i = 0; i < 4; ++i)
        {
            available[i] = nodes[i] >= 0;
            if (available[i] && part is not null) TrackControl(part(i), nodes[i]);
        }
        return new(change, available);
    }
}
