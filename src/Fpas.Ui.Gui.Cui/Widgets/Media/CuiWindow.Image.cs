// [Documentation](../../../../docs/ui/gui/text-and-display/image.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native DisplayImage operation.</summary>
    /// <param name="id">id for the native widget.</param>
    /// <param name="source">source for the native widget.</param>
    /// <param name="widthKind">widthKind for the native widget.</param>
    /// <param name="width">width for the native widget.</param>
    /// <param name="heightKind">heightKind for the native widget.</param>
    /// <param name="height">height for the native widget.</param>
    /// <param name="radius">radius for the native widget.</param>
    /// <param name="tint">Automatic, theme color, or explicit tint.</param>
    public unsafe void DisplayImage(string id, CuiMediaImage source, int widthKind, float width,
        int heightKind, float height, float radius, CuiChartColor tint)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        ValidateDistance(radius, nameof(radius));
        if (!tint.IsValid) throw new ArgumentOutOfRangeException(nameof(tint));
        using var memory = new ChartMemory();
        fixed (byte* bytes = source.Pixels)
        {
            var image = MediaSource(source, memory, bytes);
            NativeMethods.DisplayImage(context, Stable(id), in image, widthKind, width, heightKind,
                height, radius, tint.Palette, tint.Rgba);
        }
    }
}
