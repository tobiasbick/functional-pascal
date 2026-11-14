using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Audio;

/// <summary>Native call proposal.</summary>
/// <param name="Events">Native event mask.</param>
/// <param name="Muted">Proposed mute state.</param>
/// <param name="Parts">Visible mute and end controls.</param>
public sealed record CuiVoiceSessionResult(int Events, bool Muted, bool[] Parts);
