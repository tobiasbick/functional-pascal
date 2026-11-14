using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>Code editing and caret notifications.</summary>
/// <param name="Text">Edited text when it changed.</param>
/// <param name="Line">One-based caret line.</param>
/// <param name="Column">One-based caret column.</param>
/// <param name="Lines">Number of lines.</param>
/// <param name="Refused">Whether the requested edit exceeded capacity.</param>
public sealed record CuiCodeEditorEvent(string? Text, int Line, int Column, int Lines, bool Refused);
