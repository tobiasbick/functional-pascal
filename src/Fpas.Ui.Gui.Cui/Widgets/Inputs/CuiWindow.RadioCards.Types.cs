using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>A positional choice in a CUI radio-card grid.</summary>
/// <param name="Title">Literal card title.</param>
/// <param name="Description">Secondary caption, or an empty string.</param>
/// <param name="Icon">CUI icon glyph, or an empty string.</param>
/// <param name="Badge">Literal badge caption, or an empty string.</param>
public readonly record struct CuiRadioCard(string Title, string Description, string Icon, string Badge);
