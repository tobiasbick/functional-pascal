using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI option field with action.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption.</param>
    /// <param name="items">Choice captions.</param>
    /// <param name="value">Model value and native proposal.</param>
    /// <param name="icon">Optional glyph.</param>
    /// <param name="tooltip">Optional tool tooltip.</param>
    /// <param name="display">Optional display caption.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int OptionAction(string id, string label, string[] items, ref int value, string icon, string tooltip, string display,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(tooltip, nameof(tooltip));
        Utf8.Validate(display, nameof(display));
        if (items.Length is < 1 or > 32 || value < 0 || value >= items.Length)
            throw new ArgumentOutOfRangeException(nameof(value));
        foreach (string item in items) Utf8.Validate(item, nameof(items));
        int count = items.Length + 1;
        int* nodes = stackalloc int[count];
        int result;
        result = NativeMethods.OptionAction(context, Stable(id), label, items, items.Length, ref value, Optional(icon), Optional(tooltip), Optional(display), flags, focus, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
