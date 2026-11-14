using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Logical extent and minimum allocation of a native split view.</summary>
/// <param name="WidthKind">Width extent kind.</param>
/// <param name="Width">Logical width.</param>
/// <param name="HeightKind">Height extent kind.</param>
/// <param name="Height">Logical height.</param>
/// <param name="Vertical">Whether to stack the panes.</param>
/// <param name="MinimumFirst">Minimum first pane along the split axis.</param>
/// <param name="MinimumSecond">Minimum second pane along the split axis.</param>
public sealed record CuiSplitViewOptions(int WidthKind, float Width, int HeightKind, float Height,
    bool Vertical, float MinimumFirst, float MinimumSecond);

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
        CheckExtent(options.WidthKind, options.Width, nameof(options));
        CheckExtent(options.HeightKind, options.Height, nameof(options));
        CheckFeedbackNumber(options.MinimumFirst, nameof(options));
        CheckFeedbackNumber(options.MinimumSecond, nameof(options));
        if (!float.IsFinite(ratio) || ratio is < 0 or > 1) throw new ArgumentOutOfRangeException(nameof(ratio));
        return NativeMethods.SplitViewBegin(context, Stable(id), ratio, options.Vertical ? 1 : 0,
            options.MinimumFirst, options.MinimumSecond, options.WidthKind, options.Width,
            options.HeightKind, options.Height, enabled ? 1 : 0);
    }

    /// <summary>Closes the first pane, draws the divider, and opens the second pane.</summary>
    /// <param name="enabled">Whether the divider accepts input.</param>
    public void NextSplitPane(bool enabled)
    {
        CheckFrame();
        NativeMethods.SplitViewNext(context, enabled ? 1 : 0);
    }

    /// <summary>Closes the split after the second pane.</summary>
    public void EndSplitView()
    {
        CheckFrame();
        NativeMethods.SplitViewEnd(context);
    }
}
