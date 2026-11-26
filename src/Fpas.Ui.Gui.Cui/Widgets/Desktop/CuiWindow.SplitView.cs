// [Documentation](../../../../docs/ui/gui/app-chrome/split-view.md)
using Fpas.Ui.Gui.Cui.Widgets.Desktop;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens the first pane of a native split.</summary>
    /// <param name="id">Stable split identity.</param>
    /// <param name="ratio">First pane's share, from zero to one.</param>
    /// <param name="options">Native dimensions and orientation.</param>
    /// <param name="enabled">Whether the divider accepts input.</param>
    /// <returns>The proposed first-pane share.</returns>
    public float BeginSplitView(string id, float ratio, CuiSplitViewOptions options, bool enabled)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(options);
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        CheckExtent(options.HeightKind, options.Height, nameof(options));
        CheckFeedbackNumber(options.MinimumFirst, nameof(options));
        CheckFeedbackNumber(options.MinimumSecond, nameof(options));
        if (!float.IsFinite(ratio) || ratio is < 0 or > 1) throw new ArgumentOutOfRangeException(nameof(ratio));
        float proposed = NativeMethods.SplitViewBegin(context, Stable(id), ratio, options.Vertical ? 1 : 0,
            options.MinimumFirst, options.MinimumSecond, options.WidthKind, options.Width,
            options.HeightKind, options.Height, enabled ? 1 : 0);
        if (proposed < 0) throw new InvalidOperationException("The native CUI box nesting limit was reached.");
        layouts.Add(new(Layout.SplitFirst));
        return proposed;
    }

    /// <summary>Closes the first pane, draws the divider, and opens the second pane.</summary>
    /// <param name="enabled">Whether the divider accepts input.</param>
    public void NextSplitPane(bool enabled)
    {
        RequireLayout(Layout.SplitFirst);
        if (NativeMethods.SplitViewNext(context, enabled ? 1 : 0) == 0)
            throw new InvalidOperationException("Close layouts inside the first split pane before continuing.");
        layouts[^1] = new(Layout.SplitSecond);
    }

    /// <summary>Closes the split after the second pane.</summary>
    public void EndSplitView()
    {
        RequireLayout(Layout.SplitSecond);
        if (NativeMethods.SplitViewEnd(context) == 0)
            throw new InvalidOperationException("Close layouts inside the second split pane before closing the split.");
        layouts.RemoveAt(layouts.Count - 1);
    }
}
