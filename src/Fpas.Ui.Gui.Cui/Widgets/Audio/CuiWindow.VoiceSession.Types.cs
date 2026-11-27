// [Documentation](../../../../docs/ui/gui/voice/voice-session.md)
namespace Fpas.Ui.Gui.Cui.Widgets.Audio;

/// <summary>Native call proposal.</summary>
/// <param name="Events">Bit 1 reports Mute. Bit 2 reports End.</param>
/// <param name="Muted">Proposed microphone state, or false when Mute is absent.</param>
/// <param name="Parts">Presence flags in Mute, End order.</param>
public sealed record CuiVoiceSessionResult(int Events, bool Muted, bool[] Parts);
