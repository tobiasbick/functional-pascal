namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>A native dependency between indices in the caller's sorted span array.</summary>
/// <param name="From">Source span index.</param>
/// <param name="To">Destination span index.</param>
/// <param name="Tone">Native semantic tone.</param>
/// <param name="Width">Logical line width, or zero for CUI's default.</param>
/// <param name="Dashed">Whether the dependency is weak or inferred.</param>
public sealed record CuiTraceLink(int From, int To, int Tone = 0, float Width = 0, bool Dashed = false);

/// <summary>Native edits and the link under the pointer, without managed hit testing.</summary>
/// <param name="Edit">Selection, hovered span, lane and viewport proposals.</param>
/// <param name="HoveredLink">Link index, or minus one when a span owns the hover or nothing is hit.</param>
public readonly record struct CuiTraceResult(CuiDevResult Edit, int HoveredLink);
