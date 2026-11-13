using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's before-and-after image comparison.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="before">Picture shown to the left of the divider.</param>
    /// <param name="after">Picture shown to the right of the divider.</param>
    /// <param name="position">Current and proposed divider fraction.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="height">Logical height.</param>
    /// <param name="aspect">Image aspect ratio, or zero to use the first picture.</param>
    /// <param name="beforeLabel">Left caption.</param>
    /// <param name="afterLabel">Right caption.</param>
    /// <param name="enabled">Accepts native input.</param>
    /// <param name="focus">Requests native keyboard focus.</param>
    /// <returns>Whether CUI proposed a change.</returns>
    public unsafe bool ImageCompare(string id, CuiMediaImage before, CuiMediaImage after, ref float position,
        int widthKind, float width, float height, float aspect, string beforeLabel, string afterLabel, bool enabled, bool focus)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(height, nameof(height));
        CheckFeedbackNumber(aspect, nameof(aspect));
        if (!float.IsFinite(position) || position < 0 || position > 1)
            throw new ArgumentOutOfRangeException(nameof(position));
        Utf8.Validate(beforeLabel, nameof(beforeLabel));
        Utf8.Validate(afterLabel, nameof(afterLabel));
        using var memory = new ChartMemory();
        fixed (byte* left = before.Pixels, right = after.Pixels)
        {
            var a = MediaSource(before, memory, left);
            var b = MediaSource(after, memory, right);
            return NativeMethods.ImageCompare(context, Stable(id), in a, in b, ref position, widthKind, width,
                height, aspect, beforeLabel, afterLabel, enabled ? 1 : 0, focus ? 1 : 0) != 0;
        }
    }
}
