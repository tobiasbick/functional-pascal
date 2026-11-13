using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI seed field.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption.</param>
    /// <param name="seed">Native seed and proposal.</param>
    /// <param name="locked">Native lock state.</param>
    /// <param name="hasLock">Whether to show the lock.</param>
    /// <param name="display">Optional display caption.</param>
    /// <param name="random">Optional random-mode caption.</param>
    /// <param name="fixedText">Optional fixed-mode caption.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int SeedField(string id, string label, ref uint seed, ref int locked, bool hasLock, string display, string random, string fixedText,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(display, nameof(display));
        Utf8.Validate(random, nameof(random));
        Utf8.Validate(fixedText, nameof(fixedText));
        int count = 2;
        int* nodes = stackalloc int[count];
        int result;
        fixed (int* l = &locked)
            result = NativeMethods.SeedField(context, Stable(id), label, ref seed, hasLock ? l : null, Optional(display), Optional(random), Optional(fixedText), flags, focus, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
