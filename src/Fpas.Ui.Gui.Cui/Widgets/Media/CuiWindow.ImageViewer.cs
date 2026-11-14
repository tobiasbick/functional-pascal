using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native zoomable image.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="image">File or in-memory picture.</param>
    /// <param name="current">Current and proposed view.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="heightKind">Encoded height extent.</param>
    /// <param name="height">Logical height or percentage.</param>
    /// <param name="maximum">Maximum zoom ratio.</param>
    /// <param name="flags">Enabled, hide toolbar and checkerboard bits.</param>
    /// <param name="focus">Requests native keyboard focus.</param>
    /// <returns>Whether CUI proposed a change.</returns>
    public unsafe bool ImageViewer(string id, CuiMediaImage image, ref CuiImageView current, int widthKind, float width,
        int heightKind, float height, float maximum, int flags, bool focus)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        CheckFeedbackNumber(maximum, nameof(maximum));
        CheckFeedbackNumber(current.Zoom, nameof(current));
        if (!float.IsFinite(current.CenterX) || !float.IsFinite(current.CenterY))
            throw new ArgumentOutOfRangeException(nameof(current));
        using var memory = new ChartMemory();
        fixed (byte* pixels = image.Pixels)
        {
            var source = MediaSource(image, memory, pixels);
            return NativeMethods.ImageViewer(context, Stable(id), in source, ref current, widthKind,
                width, heightKind, height, maximum, flags, focus ? 1 : 0) != 0;
        }
    }
}
