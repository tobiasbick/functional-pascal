using Fpas.Ui.Gui.Cui.Interop.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Editors;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native CanvasControls operation.</summary>
    /// <param name="id">id supplied to the native operation.</param>
    /// <param name="view">view supplied to the native operation.</param>
    /// <param name="content">content supplied to the native operation.</param>
    /// <param name="steps">steps supplied to the native operation.</param>
    /// <param name="toggles">toggles supplied to the native operation.</param>
    /// <param name="states">states supplied to the native operation.</param>
    /// <param name="flags">flags supplied to the native operation.</param>
    /// <returns>The native result.</returns>
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
