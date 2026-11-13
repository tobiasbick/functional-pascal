using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI ratio slider.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="first">First caption.</param>
    /// <param name="second">Second caption.</param>
    /// <param name="value">Model value and native proposal.</param>
    /// <param name="tone">Native semantic tone.</param>
    /// <param name="step">Logical or normalized step.</param>
    /// <param name="height">Logical height.</param>
    /// <param name="widthKind">Native width encoding.</param>
    /// <param name="width">Width associated with the encoding.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int RatioSlider(string id, string first, string second, ref float value, int tone, float step, float height, int widthKind, float width,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(first, nameof(first));
        Utf8.Validate(second, nameof(second));
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(step, nameof(step));
        CheckFeedbackNumber(height, nameof(height));
        if (!float.IsFinite(value) || value is < 0 or > 1 || tone is < 0 or > 8)
            throw new ArgumentOutOfRangeException(nameof(value));
        int count = 1;
        int* nodes = stackalloc int[count];
        int result;
        result = NativeMethods.RatioSlider(context, Stable(id), first, second, ref value, tone, step, height, widthKind, width, flags, focus, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
