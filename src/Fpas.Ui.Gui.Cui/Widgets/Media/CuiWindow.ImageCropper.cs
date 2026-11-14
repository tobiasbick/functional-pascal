using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native cropper.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="image">File or in-memory picture.</param>
    /// <param name="current">Current and proposed crop in pixels.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="heightKind">Encoded height extent.</param>
    /// <param name="height">Logical height or percentage.</param>
    /// <param name="aspect">Required width divided by height, or zero.</param>
    /// <param name="minimum">Minimum side in image pixels.</param>
    /// <param name="flags">Enabled and round-mask bits.</param>
    /// <param name="focus">Requests native keyboard focus.</param>
    /// <returns>Whether CUI proposed a change.</returns>
    public unsafe bool ImageCropper(string id, CuiMediaImage image, ref CuiRectangle current, int widthKind,
        float width, int heightKind, float height, float aspect, float minimum, int flags, bool focus)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        CheckFeedbackNumber(aspect, nameof(aspect));
        CheckFeedbackNumber(minimum, nameof(minimum));
        CheckFeedbackNumber(current.Width, nameof(current));
        CheckFeedbackNumber(current.Height, nameof(current));
        if (!float.IsFinite(current.X) || !float.IsFinite(current.Y))
            throw new ArgumentOutOfRangeException(nameof(current));
        using var memory = new ChartMemory();
        fixed (byte* pixels = image.Pixels)
        {
            var source = MediaSource(image, memory, pixels);
            return NativeMethods.ImageCropper(context, Stable(id), in source, ref current, widthKind,
                width, heightKind, height, aspect, minimum, flags, focus ? 1 : 0) != 0;
        }
    }
}
