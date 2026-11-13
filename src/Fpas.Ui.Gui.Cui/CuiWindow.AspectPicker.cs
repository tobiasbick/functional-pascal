using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI aspect ratio picker.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption.</param>
    /// <param name="value">Model value and native proposal.</param>
    /// <param name="widths">Relative widths.</param>
    /// <param name="heights">Relative heights.</param>
    /// <param name="labels">Optional choice captions.</param>
    /// <param name="custom">Optional custom caption.</param>
    /// <param name="sizeBase">Short side in pixels.</param>
    /// <param name="display">Optional display caption.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int AspectPicker(string id, string label, ref int value, int[] widths, int[] heights, string[] labels, string custom, int sizeBase, string display,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(custom, nameof(custom));
        Utf8.Validate(display, nameof(display));
        if (widths.Length > 32 || widths.Length != heights.Length || widths.Length != labels.Length)
            throw new ArgumentException("Expected up to 32 aligned aspect choices.");
        int total = (widths.Length == 0 ? 6 : widths.Length) + ((flags & 2) != 0 ? 1 : 0);
        if (value < 0 || value >= total) throw new ArgumentOutOfRangeException(nameof(value));
        for (int i = 0; i < widths.Length; ++i)
        {
            Utf8.Validate(labels[i], nameof(labels));
            if (widths[i] <= 0 || heights[i] <= 0 ||
                (sizeBase == 0 ? 1080d : sizeBase) * Math.Max(widths[i], heights[i]) / Math.Min(widths[i], heights[i]) >= int.MaxValue)
                throw new ArgumentOutOfRangeException(nameof(widths));
        }
        int count = total;
        int* nodes = stackalloc int[count];
        int result;
        fixed (int* w = widths, h = heights)
            result = NativeMethods.AspectPicker(context, Stable(id), label, ref value, w, h, labels, widths.Length, Optional(custom), sizeBase, Optional(display), flags, focus, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
