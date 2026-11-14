using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Desktop;

/// <summary>Allocation and initial layout of a native docking workspace.</summary>
/// <param name="WidthKind">Width extent kind.</param>
/// <param name="Width">Logical width.</param>
/// <param name="HeightKind">Height extent kind.</param>
/// <param name="Height">Logical height.</param>
/// <param name="MinimumWidth">Minimum panel width.</param>
/// <param name="MinimumHeight">Minimum panel height.</param>
/// <param name="Shortcuts">Enable window-wide document shortcuts.</param>
/// <param name="Layout">Initial layout in the native dock1 format.</param>
public sealed record CuiDockspaceOptions(int WidthKind, float Width, int HeightKind, float Height,
    float MinimumWidth, float MinimumHeight, bool Shortcuts, string Layout);

/// <summary>Current native panel visibility and open state.</summary>
/// <param name="Visible">Whether the panel's content belongs in this frame.</param>
/// <param name="Open">Whether the panel is present in the workspace.</param>
/// <param name="HasTab">Whether a tab was drawn for the panel.</param>
public readonly record struct CuiDockPanelResult(bool Visible, bool Open, bool HasTab);
