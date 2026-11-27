// [Documentation](../../../../docs/ui/gui/voice/voice-note.md)
namespace Fpas.Ui.Gui.Cui.Widgets.Audio;

/// <summary>Native transport proposal.</summary>
/// <param name="Events">Bit 1 toggles playback, bit 8 seeks, and bit 16 changes speed.</param>
/// <param name="Position">Played seconds.</param>
/// <param name="Speed">Playback multiplier.</param>
/// <param name="Playing">Playback state.</param>
/// <param name="Parts">Present play, seek and speed controls, in that order.</param>
public sealed record CuiVoiceNoteResult(int Events, float Position, float Speed, bool Playing, bool[] Parts);
