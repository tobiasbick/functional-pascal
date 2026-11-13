using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

public sealed partial class CuiWindow
{
    /// <summary>Draws a native FlameGraph.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="spans">Ordered spans.</param>
    /// <param name="view">Model-owned state.</param>
    /// <param name="minimum">Minimum zoom range in seconds.</param>
    /// <param name="height">Logical height or native default.</param>
    /// <param name="rowHeight">Logical height per row.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Native width value.</param>
    /// <param name="flags">Native adapter options.</param>
    /// <returns>The proposed edit or observation.</returns>
    public unsafe CuiFlameRange? FlameGraph(string id, CuiFlameSpan[] spans, CuiFlameRange view, double minimum, float height, float rowHeight, int widthKind, float width, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        using var memory = new ChartMemory();
        var native = new NativeFlameSpan[spans.Length];
        for (int index = 0; index < native.Length; index++)
        {
            var item = spans[index];
            native[index] = new() { Name = memory.Text(item.Name), Start = item.Start, End = item.End, Depth = item.Depth, ColorKey = item.ColorKey };
        }
        var state = new NativeFlameRange { Start = view.Start, End = view.End, Scroll = view.Scroll, Selected = view.Selected, Zoomed = -1 };
        fixed (NativeFlameSpan* data = native)
            return NativeMethods.FlameGraph(context, Stable(id), data, native.Length, ref state, minimum, height, rowHeight, widthKind, width, flags) == 0
                ? null
                : new(state.Start, state.End, state.Scroll, state.Selected, state.Zoomed);
    }
}
