using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Desktop;

/// <summary>Native status item result.</summary>
/// <param name="Shown">Whether the item fits and was drawn.</param>
/// <param name="Clicked">Whether the item was activated.</param>
public readonly record struct CuiStatusBarResult(bool Shown, bool Clicked);
