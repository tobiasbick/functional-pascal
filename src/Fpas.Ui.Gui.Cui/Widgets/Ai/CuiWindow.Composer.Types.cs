using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Ai;

/// <summary>Native editor and recorder proposal.</summary>
/// <param name="Draft">Current or edited text.</param>
/// <param name="Events">Native event mask.</param>
/// <param name="Selected">Proposed model index.</param>
/// <param name="Removed">Removed attachment index or minus one.</param>
/// <param name="Parts">Visible editor, send, microphone and cancel controls.</param>
public sealed record CuiComposerResult(string Draft, int Events, int Selected, int Removed, bool[] Parts);
