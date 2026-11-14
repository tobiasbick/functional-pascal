using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Desktop;

/// <summary>One native document tab.</summary>
/// <param name="Key">Stable key in the strip.</param>
/// <param name="Title">Visible title.</param>
/// <param name="Icon">Optional icon glyph.</param>
/// <param name="Tooltip">Optional detailed tooltip.</param>
/// <param name="Dirty">Whether the document has unsaved changes.</param>
public sealed record CuiDocumentTab(string Key, string Title, string Icon, string Tooltip, bool Dirty);

/// <summary>Native document strip behavior.</summary>
/// <param name="Shortcuts">Whether window-wide shortcuts are enabled.</param>
/// <param name="Closable">Whether tabs can be closed.</param>
/// <param name="Reorderable">Whether tabs can be moved.</param>
public sealed record CuiDocumentTabOptions(bool Shortcuts, bool Closable, bool Reorderable);

/// <summary>One frame's document tab proposals, with absent indices set to minus one.</summary>
/// <param name="Selected">Proposed selected index after a move.</param>
/// <param name="Close">Index to close in the original order.</param>
/// <param name="MoveFrom">Source of a move.</param>
/// <param name="MoveTo">Destination of a move.</param>
public readonly record struct CuiDocumentTabChange(int Selected, int Close, int MoveFrom, int MoveTo);
