using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>Native diff selection and layout.</summary>
/// <param name="Selected">Selected change, or minus one.</param>
/// <param name="Count">Number of changes.</param>
/// <param name="Inline">Whether the layout is inline.</param>
public sealed record CuiDiffViewEvent(int Selected, int Count, bool Inline);
