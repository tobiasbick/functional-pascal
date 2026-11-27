// [Documentation](../../../../docs/ui/gui/creative-controls/status-tag.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Presents the native CUI status tag.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="label">Widget caption, or empty for the preset caption.</param>
    /// <param name="status">Native status preset.</param>
    /// <param name="icon">Replacement glyph, or empty for the preset icon.</param>
    /// <param name="tone">Native semantic tone, or zero for the preset color.</param>
    /// <param name="flags">Bits for compact size (1), removal (2), and input (4).</param>
    /// <param name="focus">Native focus part, or minus one.</param>
    /// <param name="part">Maps native parts to tracking identities.</param>
    /// <returns>One when the user requests removal, or zero otherwise.</returns>
    public unsafe int StatusTag(string id, string label, int status, string icon, int tone,
        int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        if (status is < 0 or > 7) throw new ArgumentOutOfRangeException(nameof(status));
        if (tone is < 0 or > 8) throw new ArgumentOutOfRangeException(nameof(tone));
        int node;
        int result = NativeMethods.StatusTag(context, Stable(id), Optional(label), status, Optional(icon), tone, flags, focus, &node);
        if (part is not null && node >= 0) TrackControl(part(0), node);
        return result;
    }
}
