using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Ai;

/// <summary>Native response toolbar proposal.</summary>
/// <param name="Events">Native event mask.</param>
/// <param name="Vote">Proposed vote.</param>
/// <param name="Parts">Visible action controls.</param>
public sealed record CuiResponseResult(int Events, int Vote, bool[] Parts);
