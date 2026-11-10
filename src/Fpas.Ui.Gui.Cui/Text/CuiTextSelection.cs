namespace Fpas.Ui.Gui.Cui.Text;

/// <summary>Represents a directional selection at UTF-16 grapheme boundaries.</summary>
/// <param name="Anchor">The fixed end of the selection.</param>
/// <param name="Caret">The moving end of the selection. Equal to the anchor for an insertion point.</param>
public readonly record struct CuiTextSelection(int Anchor, int Caret);
