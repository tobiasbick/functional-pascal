using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Desktop;

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
