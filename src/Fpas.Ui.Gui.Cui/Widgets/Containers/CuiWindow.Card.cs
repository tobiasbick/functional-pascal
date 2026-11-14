using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a native card around subsequent children.</summary>
    /// <param name="id">Stable card identity.</param>
    /// <param name="widthKind">Native bridge width kind.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="heightKind">Native bridge height kind.</param>
    /// <param name="height">Logical height or percentage.</param>
    /// <param name="minimum">Minimum logical width, or zero.</param>
    /// <param name="maximum">Maximum logical width, or zero.</param>
    /// <param name="padding">Inset, or zero for CUI's default.</param>
    /// <param name="paddingX">Horizontal inset override.</param>
    /// <param name="paddingY">Vertical inset override.</param>
    /// <param name="gap">Child spacing, or zero for CUI's default.</param>
    /// <param name="row">Whether children run horizontally.</param>
    /// <param name="align">Native cross-axis alignment.</param>
    /// <param name="wrap">Whether a row wraps.</param>
    /// <param name="muted">Whether to use the borderless muted surface.</param>
    /// <param name="selected">Whether to show the accent outline.</param>
    /// <param name="hoverable">Whether the card is an action.</param>
    /// <param name="enabled">Whether the card accepts activation.</param>
    /// <param name="focus">Whether to focus the card.</param>
    /// <returns>Whether the native card was activated.</returns>
    public bool BeginCard(string id, int widthKind, float width, int heightKind, float height,
        float minimum, float maximum, float padding, float paddingX, float paddingY, float gap,
        bool row, int align, bool wrap, bool muted, bool selected, bool hoverable, bool enabled, bool focus)
    {
        CheckLayoutOptions(padding, gap);
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        if (!float.IsFinite(minimum) || minimum < 0 || !float.IsFinite(maximum) ||
            maximum < 0 || maximum != 0 && maximum < minimum)
            throw new ArgumentOutOfRangeException(nameof(maximum));
        if (!float.IsFinite(paddingX) || paddingX < 0) throw new ArgumentOutOfRangeException(nameof(paddingX));
        if (!float.IsFinite(paddingY) || paddingY < 0) throw new ArgumentOutOfRangeException(nameof(paddingY));
        if (align is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(align));
        if (wrap && !row) throw new ArgumentException("Wrapping requires a horizontal card.", nameof(wrap));
        int clicked = NativeMethods.CardBegin(context, Stable(id), widthKind, width, heightKind, height,
            minimum, maximum, padding, paddingX, paddingY, gap, row ? 1 : 0, align, wrap ? 1 : 0,
            muted ? 1 : 0, selected ? 1 : 0, hoverable ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, out int node);
        layouts.Add(new(Layout.Card, Node: node));
        return clicked != 0;
    }

    /// <summary>Closes the innermost native card and makes it the last widget.</summary>
    public void EndCard()
    {
        var card = RequireLayout(Layout.Card);
        NativeMethods.CardEnd(context, card.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
