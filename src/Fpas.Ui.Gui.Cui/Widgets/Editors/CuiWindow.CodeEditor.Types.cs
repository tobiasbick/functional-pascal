namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>Code editing and caret notifications.</summary>
/// <param name="Text">Edited text when it changed.</param>
/// <param name="Line">One-based caret line.</param>
/// <param name="Column">One-based visual caret column, including tab stops.</param>
/// <param name="Lines">Number of lines.</param>
/// <param name="Refused">Whether the requested edit exceeded capacity.</param>
/// <param name="Cursor">Caret offset in UTF-16 code units at a scalar boundary.</param>
/// <param name="Anchor">Selection anchor in UTF-16 code units.</param>
/// <param name="Matches">Number of native find matches.</param>
/// <param name="MatchIndex">One-based selected find match, or zero.</param>
/// <param name="Focused">Whether the editor or its find bar owns the keyboard.</param>
public sealed record CuiCodeEditorEvent(string? Text, int Line, int Column, int Lines, bool Refused,
    int Cursor = 0, int Anchor = 0, int Matches = 0, int MatchIndex = 0, bool Focused = false);

/// <summary>A highlight range at UTF-16 scalar boundaries in the model text.</summary>
/// <param name="Start">Inclusive start offset.</param>
/// <param name="End">Exclusive end offset.</param>
/// <param name="Color">Zero for accent; one through six for native series colors.</param>
public readonly record struct CuiTextMark(int Start, int End, int Color = 0);
