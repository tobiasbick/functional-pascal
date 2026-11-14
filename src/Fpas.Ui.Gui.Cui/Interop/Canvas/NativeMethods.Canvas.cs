using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_drawing_canvas", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DrawingCanvas(nint context, string id, int widthKind, float width, int heightKind, float height, CuiDrawingShape* shapes, int count, int enabled, out float x, out float y, out float w, out float h);
}
