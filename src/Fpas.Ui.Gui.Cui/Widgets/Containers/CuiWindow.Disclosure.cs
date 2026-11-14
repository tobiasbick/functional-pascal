using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Begins a native accordion or expandable card; only close a drawn body.</summary>
    /// <param name="id">Stable header identity, also used for snapshots.</param>
    /// <param name="title">Literal header caption.</param>
    /// <param name="card">Whether to use the card rather than accordion widget.</param>
    /// <param name="open">Model-owned expansion.</param>
    /// <param name="enabled">Whether the header accepts input; the host admits each body control separately.</param>
    /// <param name="focus">Whether to request header focus.</param>
    /// <param name="description">Card supporting text.</param>
    /// <param name="icon">Optional card glyph.</param>
    /// <param name="badge">Optional card badge.</param>
    /// <param name="tone">Native badge tone from zero through eight.</param>
    /// <returns>Draw-body bit 1, changed bit 2 and proposed-open bit 4.</returns>
    public int BeginDisclosure(string id, string title, bool card, bool open, bool enabled,
        bool focus, string description = "", string icon = "", string badge = "", int tone = 0)
    {
        CheckFrame();
        Utf8.Validate(title, nameof(title));
        Utf8.Validate(description, nameof(description));
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(badge, nameof(badge));
        if (tone is < 0 or > 8) throw new ArgumentOutOfRangeException(nameof(tone));
        if (layouts.Count(static scope => scope.Kind is Layout.Accordion or Layout.ExpandableCard) >= 32)
            throw new InvalidOperationException("CUI supports up to thirty-two nested disclosure bodies.");
        int result = NativeMethods.DisclosureBegin(context, Stable(id), title, card ? 1 : 0,
            open ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, description, icon, badge, tone,
            out int node, out int header);
        if (result < 0) throw new InvalidOperationException("CUI's disabled-scope stack is full.");
        TrackControl(id, header);
        if ((result & 1) != 0) layouts.Add(new(card ? Layout.ExpandableCard : Layout.Accordion, node));
        return result;
    }

    /// <summary>Ends the current accordion or expandable-card body and restores its layout target.</summary>
    public void EndDisclosure()
    {
        CheckFrame();
        if (layouts.Count == 0 || layouts[^1].Kind is not (Layout.Accordion or Layout.ExpandableCard))
            throw new InvalidOperationException("Close the innermost disclosure body after its child layouts.");
        var scope = layouts[^1];
        NativeMethods.DisclosureEnd(context, scope.Kind == Layout.ExpandableCard ? 1 : 0, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Draws CUI-owned expandable text and tracks its optional native link separately.</summary>
    /// <param name="id">Stable text identity.</param>
    /// <param name="text">Literal wrapping text.</param>
    /// <param name="lines">Positive collapsed line count.</param>
    /// <param name="size">Font size, or zero for CUI's body size.</param>
    /// <param name="color">Text color in low-byte-first RGBA, or zero for the theme.</param>
    /// <param name="fadeTo">Fade-surface color in low-byte-first RGBA, or zero for the theme.</param>
    /// <param name="enabled">Whether the link accepts input.</param>
    /// <param name="focus">Whether to request link focus, when it exists.</param>
    /// <param name="toggleId">Snapshot identity of the optional link.</param>
    /// <returns>Changed bit 1, expanded bit 2 and link-present bit 4.</returns>
    public int ExpandableText(string id, string text, int lines, float size,
        uint color, uint fadeTo, bool enabled, bool focus, string toggleId)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        ArgumentException.ThrowIfNullOrEmpty(toggleId);
        ArgumentOutOfRangeException.ThrowIfLessThan(lines, 1);
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        int result = NativeMethods.ExpandableText(context, Stable(id), text, lines, size,
            (color != 0 ? 1 : 0) | (fadeTo != 0 ? 2 : 0), color, fadeTo,
            enabled ? 1 : 0, focus ? 1 : 0, out int toggle);
        if (toggle >= 0) TrackControl(toggleId, toggle);
        TrackControl(id, NativeMethods.LastNode(context));
        return result | (toggle >= 0 ? 4 : 0);
    }
}
