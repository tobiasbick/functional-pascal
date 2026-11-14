using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Audio;

/// <summary>Transport proposal and snapshot part availability.</summary>
/// <param name="Events">Native toggle, previous, next and seek bits.</param>
/// <param name="Parts">Play, previous, next and seek node availability.</param>
public readonly record struct CuiPlayerResult(int Events, bool[] Parts);
