using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a native card deck.</summary>
    /// <param name="id">Stable deck identity.</param>
    /// <param name="count">Number of supplied cards.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="height">One card's height.</param>
    /// <param name="visible">Number of visible cards.</param>
    /// <param name="enabled">Accepts deck input.</param>
    /// <param name="focus">Requests focus for the top card.</param>
    public void BeginCardStack(string id, int count, int widthKind, float width, float height, int visible, bool enabled, bool focus)
    {
        CheckFrame();
        if (count < 0 || visible < 0) throw new ArgumentOutOfRangeException(nameof(count));
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(height, nameof(height));
        int root = NativeMethods.CardStackBegin(context, Stable(id), count, widthKind, width, height, visible,
            enabled ? 1 : 0, focus ? 1 : 0);
        layouts.Add(new(Layout.CardStack, Node: root, Data: enabled && count > 0));
    }

    /// <summary>Opens a native card in the deck.</summary>
    /// <param name="depth">Zero-based position from the top.</param>
    public void BeginCardStackItem(int depth)
    {
        RequireLayout(Layout.CardStack);
        NativeMethods.CardStackItemBegin(context, depth);
        layouts.Add(new(Layout.CardStackItem));
    }

    /// <summary>Closes the current card.</summary>
    public void EndCardStackItem()
    {
        RequireLayout(Layout.CardStackItem);
        NativeMethods.CardStackItemEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Closes the deck and reports a completed swipe.</summary>
    /// <param name="topId">Optional inspection identity of the top card.</param>
    /// <returns>Minus one for left, one for right, or zero.</returns>
    public int EndCardStack(string? topId)
    {
        var scope = RequireLayout(Layout.CardStack);
        int result = NativeMethods.CardStackEnd(context, scope.Node, (bool)scope.Data! ? 1 : 0, out int top);
        layouts.RemoveAt(layouts.Count - 1);
        if (top >= 0 && topId is not null) TrackControl(topId, top);
        return result;
    }
}
