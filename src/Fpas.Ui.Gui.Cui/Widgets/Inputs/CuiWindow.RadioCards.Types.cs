// [Documentation](../../../../docs/ui/gui/selection/radio-cards.md)
namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>A positional choice in a CUI radio-card grid.</summary>
/// <param name="Title">Literal card title.</param>
/// <param name="Description">Secondary caption, or null to omit the description.</param>
/// <param name="Icon">CUI icon glyph, or null to omit the icon slot.</param>
/// <param name="Badge">Literal badge caption, or null to omit the badge.</param>
public readonly record struct CuiRadioCard(string Title, string? Description, string? Icon, string? Badge);
