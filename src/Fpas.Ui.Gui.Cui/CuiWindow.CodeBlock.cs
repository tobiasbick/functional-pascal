using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native syntax-highlighted code block.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="code">Literal source text.</param>
    /// <param name="language">Optional highlighting language.</param>
    /// <param name="title">Optional header caption.</param>
    /// <param name="first">First highlighted line, or zero.</param>
    /// <param name="last">Last highlighted line, or zero.</param>
    /// <param name="height">Maximum scroll height, or zero.</param>
    /// <param name="size">Font size, or zero.</param>
    /// <param name="flags">Line numbers, hidden header and input permission.</param>
    /// <param name="copyId">Optional identity for the copy button.</param>
    /// <returns>Whether the native copy button exists.</returns>
    public bool CodeBlock(string id, string code, string language, string title, int first,
        int last, float height, float size, int flags, string? copyId)
    {
        CheckFrame();
        Utf8.Validate(code, nameof(code));
        Utf8.Validate(language, nameof(language));
        Utf8.Validate(title, nameof(title));
        CheckFeedbackNumber(height, nameof(height));
        CheckFeedbackNumber(size, nameof(size));
        if (first < 0 || last < 0 || last != 0 && (first == 0 || last < first))
            throw new ArgumentException("Invalid highlight range.", nameof(first));
        NativeMethods.CodeBlock(context, Stable(id), code, Optional(language), Optional(title),
            first, last, height, size, flags, out int copy);
        if (copy >= 0 && copyId is not null) TrackControl(copyId, copy);
        return copy >= 0;
    }
}
