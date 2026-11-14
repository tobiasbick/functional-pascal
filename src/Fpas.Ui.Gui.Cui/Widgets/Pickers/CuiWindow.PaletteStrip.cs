using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI palette strip.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption.</param>
    /// <param name="colors">Packed RGBA colors.</param>
    /// <param name="locked">Native lock state.</param>
    /// <param name="seed">Native seed and proposal.</param>
    /// <param name="hasSeed">Whether the native widget owns this shuffle proposal.</param>
    /// <param name="display">Optional display caption.</param>
    /// <param name="height">Logical height.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int PaletteStrip(string id, string label, uint[] colors, int[] locked, ref uint seed, bool hasSeed, string display, float height,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(display, nameof(display));
        if (colors.Length is < 1 or > 16 || colors.Length != locked.Length)
            throw new ArgumentException("Expected one to sixteen swatches with matching lock states.");
        CheckFeedbackNumber(height, nameof(height));
        int count = colors.Length + 1;
        int* nodes = stackalloc int[count];
        int result;
        fixed (uint* c = colors, s = &seed)
        fixed (int* l = locked)
            result = NativeMethods.PaletteStrip(context, Stable(id), label, c, l, colors.Length, hasSeed ? s : null, Optional(display), height, flags, focus, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
