using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Audio;

/// <summary>Native transport proposal.</summary>
/// <param name="Events">Native event mask.</param>
/// <param name="Position">Played seconds.</param>
/// <param name="Speed">Playback multiplier.</param>
/// <param name="Playing">Playback state.</param>
/// <param name="Parts">Visible play, seek and speed controls.</param>
public sealed record CuiVoiceNoteResult(int Events, float Position, float Speed, bool Playing, bool[] Parts);
