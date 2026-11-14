using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>The rows CUI wants to realize, including overscan.</summary>
/// <param name="First">First inclusive row.</param>
/// <param name="Last">Last exclusive row.</param>
public readonly record struct CuiVirtualRange(int First, int Last);
