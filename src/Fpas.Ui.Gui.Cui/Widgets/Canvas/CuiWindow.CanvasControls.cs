// [Documentation](../../../../docs/ui/gui/canvas-and-bars/canvas-controls.md)
using Fpas.Ui.Gui.Cui.Interop.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Editors;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Overlays zoom and toggle controls on the previous component, preserving its layout target.</summary>
    /// <param name="id">Stable controls identity.</param>
    /// <param name="view">Current pan and zoom, updated with native proposals.</param>
    /// <param name="content">Content bounds in canvas points for fitting; empty bounds only report a fit request.</param>
    /// <param name="steps">Ascending positive zoom presets; an empty array uses CUI's presets from 25% to 400%.</param>
    /// <param name="toggles">Icons and tooltips for additional controls, in display order.</param>
    /// <param name="states">Current and proposed toggle values, one per toggle.</param>
    /// <param name="flags">Bits: 1 accepts input, 2 hides fit, 4 places the rail at the reading-direction start edge.</param>
    /// <returns>Bits: 1 zoom changed, 2 fit requested, 4 a toggle changed; zero when idle or disabled.</returns>
    /// <remarks>The view can advance toward a previous zoom target even when this call returns zero.</remarks>
    public unsafe int CanvasControls(string id, ref CuiNodeView view, CuiRectangle content, float[] steps, CuiCanvasToggle[] toggles, bool[] states, int flags)
    {
        CheckFrame();
        if (states.Length != toggles.Length) throw new ArgumentException("Each toggle requires a state.", nameof(states));
        using var memory = new ChartMemory();
        var native = Array.ConvertAll(toggles, toggle => new NativeCanvasToggle
        {
            Icon = memory.Text(toggle.Icon), Label = memory.Text(toggle.Label)
        });
        byte[] values = Array.ConvertAll(states, value => value ? (byte)1 : (byte)0);
        fixed (float* zoom = steps)
        fixed (byte* on = values)
        {
            int result = NativeMethods.CanvasControls(context, Stable(id), ref view, content, zoom, steps.Length,
                memory.Copy<NativeCanvasToggle>(native), on, toggles.Length, flags);
            if ((result & 4) != 0)
                for (int i = 0; i < values.Length; i++) states[i] = values[i] != 0;
            return result;
        }
    }
}
