using Fpas.Ui.Gui.Cui.Widgets.Charts;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Widgets.Motion;

/// <summary>Options common to CUI's animated text widgets.</summary>
/// <param name="Font">Built-in typeface.</param>
/// <param name="WidthKind">Encoded width extent.</param>
/// <param name="Width">Logical width or percentage.</param>
/// <param name="Color">Semantic or explicit color.</param>
/// <param name="Size">Font size in logical points, or zero.</param>
/// <param name="Speed">Effect speed or zero.</param>
/// <param name="Delay">Seconds before the effect begins.</param>
/// <param name="NoWrap">Whether to keep the text on one line.</param>
/// <param name="Caret">Whether to show the typewriter caret.</param>
public readonly record struct CuiMotionText(int Font, int WidthKind, float Width, CuiChartColor Color,
    float Size, float Speed = 0, float Delay = 0, bool NoWrap = false, bool Caret = false);

[StructLayout(LayoutKind.Sequential)]
internal struct NativeMotionText
{
    internal int Font, WidthKind, Palette, Flags;
    internal uint Rgba;
    internal float Size, Width, Speed, Delay;
}
