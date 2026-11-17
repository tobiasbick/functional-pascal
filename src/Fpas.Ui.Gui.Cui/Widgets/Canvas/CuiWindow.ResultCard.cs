using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws an activatable image card with a window-owned native image cache.</summary>
    /// <param name="id">Stable card identity.</param>
    /// <param name="image">Immutable RGBA pixels or an image filename.</param>
    /// <param name="title">Visible card title.</param>
    /// <param name="caption">Optional secondary caption.</param>
    /// <param name="badge">Optional badge caption.</param>
    /// <param name="tone">Native semantic tone ordinal for the badge.</param>
    /// <param name="widthKind">Content=0, fill=1, logical points=2, percentage=3.</param>
    /// <param name="width">Nonnegative logical width or percentage, according to the extent kind.</param>
    /// <param name="aspect">Image width divided by height; zero uses the native default.</param>
    /// <param name="enabled">Whether the card accepts input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <returns>Whether the enabled card was activated.</returns>
    public unsafe bool ResultCard(string id, CuiMediaImage image, string title, string? caption, string? badge, int tone, int widthKind, float width, float aspect, bool enabled, bool focus)
    {
        CheckFrame();
        using var memory = new ChartMemory();
        fixed (byte* pixels = image.Pixels)
        {
            var source = MediaSource(image, memory, pixels);
            return NativeMethods.ResultCard(context, Stable(id), in source, title, caption, badge, tone,
                widthKind, width, aspect, enabled ? 1 : 0, focus ? 1 : 0) != 0;
        }
    }
}
