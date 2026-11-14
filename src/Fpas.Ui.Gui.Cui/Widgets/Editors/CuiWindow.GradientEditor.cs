using Fpas.Ui.Gui.Cui.Widgets.Editors;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native GradientEditor.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="stops">Ordered ramp stops.</param>
    /// <param name="capacity">Maximum item count.</param>
    /// <param name="selected">Selected index.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Native width value.</param>
    /// <param name="height">Logical height or native default.</param>
    /// <param name="enabled">Accept pointer and key input.</param>
    /// <returns>The proposed edit or observation.</returns>
    public unsafe CuiGradientEdit? GradientEditor(string id, CuiGradientStop[] stops, int capacity, int selected, int widthKind, float width, float height, bool enabled)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        if (capacity < stops.Length) throw new ArgumentOutOfRangeException(nameof(capacity));
        var buffer = new CuiGradientStop[capacity];
        stops.CopyTo(buffer, 0);
        int count = stops.Length;
        fixed (CuiGradientStop* data = buffer)
            return NativeMethods.GradientEditor(context, Stable(id), data, ref count, capacity, ref selected, out int added, out int removed, widthKind, width, height, enabled ? 1 : 0) == 0
                ? null
                : new(buffer[..count], selected, added, removed);
    }
}
