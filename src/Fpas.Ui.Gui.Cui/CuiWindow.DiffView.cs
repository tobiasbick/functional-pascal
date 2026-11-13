using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Native diff selection and layout.</summary>
/// <param name="Selected">Selected change, or minus one.</param>
/// <param name="Count">Number of changes.</param>
/// <param name="Inline">Whether the layout is inline.</param>
public sealed record CuiDiffViewEvent(int Selected, int Count, bool Inline);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native DiffView widget.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="before">Previous file contents.</param>
    /// <param name="after">Updated file contents.</param>
    /// <param name="current">Selected change, or minus one.</param>
    /// <param name="language">Native syntax language.</param>
    /// <param name="oldTitle">Optional caption for the previous file.</param>
    /// <param name="newTitle">Optional caption for the updated file.</param>
    /// <param name="contextLines">Unchanged lines retained around changes.</param>
    /// <param name="widthKind">Native extent kind.</param>
    /// <param name="width">Extent value for fixed and percentage widths.</param>
    /// <param name="height">Logical height, or the native auto/fill sentinel.</param>
    /// <param name="size">Logical font size, or zero for the default.</param>
    /// <param name="flags">Widget-specific input and rendering flags.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public unsafe CuiDiffViewEvent? DiffView(string id, string before, string after, int current, string language, string oldTitle, string newTitle, int contextLines, int widthKind, float width, float height, float size, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        int changed = NativeMethods.DiffView(context, Stable(id), before, after, current, language,
            OptionalAi(oldTitle), OptionalAi(newTitle), contextLines, widthKind, width, height, size, flags,
            out int selected, out int count, out int inlineMode);
        return changed == 0 ? null : new(selected, count, inlineMode != 0);
    }
}
