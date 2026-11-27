// [Documentation](../../../../docs/ui/gui/canvas-and-bars/painting.md)
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Interop.Media;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_font_scale")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial float CanvasFontScale(nint ui);
    [LibraryImport(Library, EntryPoint = "cui_time")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial double CanvasTime(nint ui);
    [LibraryImport(Library, EntryPoint = "cui_delta_time")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial float CanvasDeltaTime(nint ui);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_animate", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial float CanvasAnimate(nint ui, uint canvas, string id, float target, float duration, float bounce);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_color")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint CanvasColor(nint ui, int token);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_primitive", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void CanvasPrimitive(nint ui, int kind, float x, float y, float* values,
        uint color, uint second, string? text, CuiCorners radii);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_points")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void CanvasPoints(nint ui, float x, float y, CuiPoint* points, int count, float thickness, uint color, int kind);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_measure", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiTextMetrics CanvasMeasure(nint ui, int font, float size, string text);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_push_clip")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int CanvasPushClip(nint ui, float x, float y, float width, float height);
    [LibraryImport(Library, EntryPoint = "cui_pop_clip")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CanvasPopClip(nint ui);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_image")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CanvasImage(nint ui, uint images, int index, in NativeMediaSource image,
        float x, float y, float width, float height, CuiCorners radii, CuiImageRegion uv, uint tint);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_images_finish")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CanvasImagesFinish(nint ui, uint images, int count);
}
