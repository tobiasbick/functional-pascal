using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a group of placeholders that share native animation.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="gap">Spacing, or zero for the native default.</param>
    /// <param name="dither">Whether to draw an ordered-dot sweep.</param>
    public void BeginSkeletonGroup(string id, int widthKind, float width, float gap, bool dither)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(gap, nameof(gap));
        int node = NativeMethods.SkeletonGroupBegin(context, Stable(id), widthKind, width, gap, dither ? 1 : 0);
        layouts.Add(new(Layout.SkeletonGroup, Node: node));
    }

    /// <summary>Closes the innermost native skeleton group.</summary>
    public void EndSkeletonGroup()
    {
        var group = RequireLayout(Layout.SkeletonGroup);
        NativeMethods.SkeletonGroupEnd(context, group.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
