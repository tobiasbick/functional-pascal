using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>Native expansion and activation proposals.</summary>
/// <param name="Open">Whether CUI opened the children.</param>
/// <param name="Clicked">Whether the native row was activated.</param>
public readonly record struct CuiTreeResult(bool Open, bool Clicked);
