// [Documentation](../../../../docs/ui/gui/code/markdown.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Display;

/// <summary>A link or task activation.</summary>
/// <param name="Link">Activated URL.</param>
/// <param name="Offset">UTF-8 task offset, or minus one.</param>
/// <param name="Line">One-based task line.</param>
/// <param name="Checked">Proposed task state.</param>
/// <param name="Hovered">URL under the pointer.</param>
/// <param name="HoverChanged">Whether the hover target changed.</param>
/// <param name="Height">Logical height from the latest layout, or zero before layout.</param>
public sealed record CuiMarkdownEvent(string? Link, int Offset, int Line, bool Checked, string? Hovered, bool HoverChanged, float Height = 0);
