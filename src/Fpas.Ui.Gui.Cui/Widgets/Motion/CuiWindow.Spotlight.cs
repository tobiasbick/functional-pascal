// [Documentation](../../../../docs/ui/gui/motion/spotlight-card.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a native spotlight container.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="heightKind">Encoded height extent.</param>
    /// <param name="height">Logical height or percentage.</param>
    /// <param name="padding">Logical inset.</param>
    /// <param name="gap">Logical spacing.</param>
    /// <param name="color">Semantic or explicit color.</param>
    /// <param name="beam">Whether to draw the traveling beam.</param>
    /// <param name="acceptHover">Whether pointer proximity lights the card.</param>
    public void BeginSpotlight(string id, int widthKind, float width, int heightKind,
        float height, float padding, float gap, CuiChartColor color, bool beam, bool acceptHover = true)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        CheckFeedbackNumber(padding, nameof(padding));
        CheckFeedbackNumber(gap, nameof(gap));
        CheckChartColor(color);
        int flags = (beam ? 1 : 0) | (acceptHover ? 2 : 0);
        int node = NativeMethods.SpotlightBegin(context, Stable(id), widthKind, width, heightKind, height,
            padding, gap, color.Palette, color.Rgba, flags);
        layouts.Add(new(Layout.Spotlight, Node: node));
    }

    /// <summary>Closes the innermost spotlight container.</summary>
    public void EndSpotlight()
    {
        var scope = RequireLayout(Layout.Spotlight);
        NativeMethods.SpotlightEnd(context, scope.Node);
        layouts.RemoveAt(layouts.Count - 1);
    }
}
