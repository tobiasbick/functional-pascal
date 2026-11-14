using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Invokes the native DrawingCanvas operation.</summary>
    /// <param name="id">id for the native widget.</param>
    /// <param name="widthKind">widthKind for the native widget.</param>
    /// <param name="width">width for the native widget.</param>
    /// <param name="heightKind">heightKind for the native widget.</param>
    /// <param name="height">height for the native widget.</param>
    /// <param name="shapes">shapes for the native widget.</param>
    /// <param name="enabled">enabled for the native widget.</param>
    /// <param name="x">x for the native widget.</param>
    /// <param name="y">y for the native widget.</param>
    /// <param name="w">w for the native widget.</param>
    /// <param name="h">h for the native widget.</param>
    /// <returns>The native widget result.</returns>
    public unsafe bool DrawingCanvas(string id, int widthKind, float width, int heightKind, float height, CuiDrawingShape[] shapes, bool enabled, out float x, out float y, out float w, out float h)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckExtent(heightKind, height, nameof(height));
        fixed (CuiDrawingShape* data = shapes)
            return NativeMethods.DrawingCanvas(context, Stable(id), widthKind, width, heightKind, height,
                data, shapes.Length, enabled ? 1 : 0, out x, out y, out w, out h) != 0;
    }
}
