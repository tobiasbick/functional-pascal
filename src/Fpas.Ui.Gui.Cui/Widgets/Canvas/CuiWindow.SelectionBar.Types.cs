// [Documentation](../../../../docs/ui/gui/canvas-and-bars/selection-bar.md)
namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>An action offered for selected items.</summary>
/// <param name="Icon">CUI icon glyph, or an empty string for a text-only action.</param>
/// <param name="Label">Visible action caption.</param>
/// <param name="Tone">Native semantic tone ordinal.</param>
public sealed record CuiSelectionAction(string Icon, string Label, int Tone);
