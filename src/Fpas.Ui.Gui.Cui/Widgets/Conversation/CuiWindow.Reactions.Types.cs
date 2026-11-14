using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Conversation;

/// <summary>One model-owned native reaction.</summary>
/// <param name="Icon">CUI icon glyph.</param>
/// <param name="Count">Current nonnegative count.</param>
/// <param name="Active">Whether the current user has reacted.</param>
public readonly record struct CuiReaction(string Icon, int Count, bool Active);

/// <summary>A reaction change proposed by CUI.</summary>
/// <param name="Index">Zero-based changed item, or minus one.</param>
/// <param name="Count">Proposed count when an item changed.</param>
/// <param name="Active">Proposed activation when an item changed.</param>
public readonly record struct CuiReactionChange(int Index, int Count, bool Active);
