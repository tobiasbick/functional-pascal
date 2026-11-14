using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI step slider.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption.</param>
    /// <param name="value">Model value and native proposal.</param>
    /// <param name="minimum">Lower endpoint.</param>
    /// <param name="maximum">Upper endpoint.</param>
    /// <param name="labels">Optional choice captions.</param>
    /// <param name="prefix">Literal prefix for the integer.</param>
    /// <param name="suffix">Literal suffix for the integer.</param>
    /// <param name="display">Optional display caption.</param>
    /// <param name="tool">Optional tool glyph.</param>
    /// <param name="tooltip">Optional tool tooltip.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int StepSlider(string id, string label, ref int value, int minimum, int maximum, string[] labels, string prefix, string suffix, string display, string tool, string tooltip,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(display, nameof(display));
        Utf8.Validate(prefix, nameof(prefix));
        Utf8.Validate(suffix, nameof(suffix));
        Utf8.Validate(tool, nameof(tool));
        Utf8.Validate(tooltip, nameof(tooltip));
        long range = (long)maximum - minimum;
        if (range is < 1 or > 16_777_216 || value < minimum || value > maximum)
            throw new ArgumentOutOfRangeException(nameof(value));
        if (labels.Length != 0 && labels.Length != range + 1)
            throw new ArgumentException("Expected a label per stop.", nameof(labels));
        foreach (string caption in labels) Utf8.Validate(caption, nameof(labels));
        string? format = prefix.Length == 0 && suffix.Length == 0 ? null :
            prefix.Replace("%", "%%", StringComparison.Ordinal) + "%d" + suffix.Replace("%", "%%", StringComparison.Ordinal);
        int count = 2;
        int* nodes = stackalloc int[count];
        int result;
        result = NativeMethods.StepSlider(context, Stable(id), label, ref value, minimum, maximum, labels, labels.Length, format, Optional(display), Optional(tool), Optional(tooltip), flags, focus, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
