using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI status tag.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption.</param>
    /// <param name="status">Native status preset.</param>
    /// <param name="icon">Optional glyph.</param>
    /// <param name="tone">Native semantic tone.</param>
    /// <param name="flags">Native appearance and input bits.</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>The native change and action flags.</returns>
    public unsafe int StatusTag(string id, string label, int status, string icon, int tone,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        if (status is < 0 or > 7 || tone is < 0 or > 8) throw new ArgumentOutOfRangeException(nameof(status));
        int count = 1;
        int* nodes = stackalloc int[count];
        int result;
        result = NativeMethods.StatusTag(context, Stable(id), Optional(label), status, Optional(icon), tone, flags, focus, nodes);
        if (part is not null)
            for (int i = 0; i < count; ++i)
                if (nodes[i] >= 0) TrackControl(part(i), nodes[i]);
        return result;
    }
}
