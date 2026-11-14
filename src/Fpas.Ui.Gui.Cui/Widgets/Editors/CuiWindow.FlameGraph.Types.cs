using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>A timed frame span.</summary>
/// <param name="Name">Display name.</param>
/// <param name="Start">Start seconds.</param>
/// <param name="End">End seconds.</param>
/// <param name="Depth">Call depth.</param>
/// <param name="ColorKey">Optional color key.</param>
public sealed record CuiFlameSpan(string Name, double Start, double End, int Depth, uint ColorKey);

/// <summary>Graph view and selection.</summary>
/// <param name="Start">Visible start.</param>
/// <param name="End">Visible end.</param>
/// <param name="Scroll">Vertical offset.</param>
/// <param name="Selected">Selected index.</param>
/// <param name="Zoomed">Zoom target.</param>
public sealed record CuiFlameRange(double Start, double End, float Scroll, int Selected, int Zoomed = -1);

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeFlameSpan
{
    internal byte* Name;
    internal double Start, End;
    internal int Depth;
    internal uint ColorKey;
}

[StructLayout(LayoutKind.Sequential)]
internal struct NativeFlameRange
{
    internal double Start, End;
    internal float Scroll;
    internal int Selected, Zoomed;
}
