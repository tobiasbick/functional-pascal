using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Overlays;

/// <summary>Native tour navigation and visible action slots.</summary>
/// <param name="Step">Proposed step, minus one when closed.</param>
/// <param name="Event">None zero, next one, back two, done three, skipped four.</param>
/// <param name="VisibleParts">Close zero, Back one and Next/Done two when present.</param>
public readonly record struct CuiTourResult(int Step, int Event, int[] VisibleParts);
