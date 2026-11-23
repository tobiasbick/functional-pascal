// [Documentation](../../../../docs/ui/gui/canvas-and-bars/painting.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using System.Runtime.CompilerServices;
using System.Runtime.ExceptionServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private readonly List<GCHandle> canvasPainters = [];
    private bool paintingCanvas;

    private sealed class CanvasPaint(CuiWindow owner, Action<CuiCanvasPainter> paint)
    {
        internal readonly CuiWindow Owner = owner;
        internal readonly Action<CuiCanvasPainter> Paint = paint;
        internal ExceptionDispatchInfo? Failure;
    }

    private nint RetainCanvasPaint(Action<CuiCanvasPainter>? paint)
    {
        if (paint is null) return 0;
        var root = GCHandle.Alloc(new CanvasPaint(this, paint));
        canvasPainters.Add(root);
        return GCHandle.ToIntPtr(root);
    }

    private ExceptionDispatchInfo? ReleaseFrameCallbacks()
    {
        var failure = ReleaseAxisLabels();
        foreach (var root in canvasPainters)
        {
            failure ??= ((CanvasPaint)root.Target!).Failure;
            root.Free();
        }
        canvasPainters.Clear();
        return failure;
    }

    private static unsafe nint CanvasPaintFunction(nint user) => user == 0 ? 0 :
        (nint)(delegate* unmanaged[Cdecl]<nint, nint, CuiRectangle, uint, int, void>)&PaintCanvas;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void PaintCanvas(nint user, nint context, CuiRectangle area, uint images, int state)
    {
        var callback = (CanvasPaint)GCHandle.FromIntPtr(user).Target!;
        if (callback.Failure is not null) return;
        var painter = new CuiCanvasPainter(context, area, images, state);
        try
        {
            callback.Owner.paintingCanvas = true;
            callback.Paint(painter);
        }
        catch (Exception error) { callback.Failure = ExceptionDispatchInfo.Capture(error); }
        finally
        {
            painter.Complete();
            callback.Owner.paintingCanvas = false;
        }
    }

    internal static unsafe void PaintImage(nint context, uint images, int index, CuiMediaImage source,
        float x, float y, float width, float height, float radius, uint tint)
    {
        using var memory = new ChartMemory();
        fixed (byte* bytes = source.Pixels)
        {
            var image = MediaSource(source, memory, bytes);
            NativeMethods.CanvasImage(context, images, index, in image, x, y, width, height, radius, tint);
        }
    }
}
