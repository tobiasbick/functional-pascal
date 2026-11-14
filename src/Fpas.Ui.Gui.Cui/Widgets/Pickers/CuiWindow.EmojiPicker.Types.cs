using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Pickers;

/// <summary>One native emoji picker frame's selection and persistence data.</summary>
/// <param name="Recent">Recent emoji in CUI's space-separated order.</param>
/// <param name="Tone">Selected skin tone from zero through five.</param>
/// <param name="Emoji">The selected UTF-16 text, or null.</param>
/// <param name="VisibleParts">The indices of currently tracked native controls.</param>
public readonly record struct CuiEmojiPickerResult(string Recent, int Tone, string? Emoji, int[] VisibleParts);
