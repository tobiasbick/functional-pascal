// [Documentation](../../../../docs/ui/gui/text-and-display/canvas.md)
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [StructLayout(LayoutKind.Sequential)]
    internal struct CanvasEvent
    {
        internal int Kind;
        internal float X, Y, Dx, Dy;
        internal int Button, Key, Mods, Finger;
        internal byte Down, Focused;
        internal nint Text;
    }

    [LibraryImport(Library, EntryPoint = "fpas_cui_input_event_layout")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CanvasEventLayout(out int size, out int finger, out int down, out int text);
    [LibraryImport(Library, EntryPoint = "cui_set_input_hook")]
    internal static unsafe partial void CanvasHook(nint ui, delegate* unmanaged[Cdecl]<nint, CanvasEvent*, nint, byte> hook, nint user);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_dispatch", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial uint CanvasDispatch(nint ui, int kind, int key, int mods, int down, string? text, out int repeated);
    [LibraryImport(Library, EntryPoint = "cui_mouse_pos")]
    internal static partial CuiPoint CanvasPointer(nint ui);
    [LibraryImport(Library, EntryPoint = "cui_mouse_down")]
    internal static partial byte CanvasButtonDown(nint ui, int button);
    [LibraryImport(Library, EntryPoint = "fpas_cui_canvas_pointer_focus")]
    internal static partial int CanvasPointerFocus(nint ui, float x, float y);
    [LibraryImport(Library, EntryPoint = "cui_focused_id")]
    internal static partial uint CanvasFocusedId(nint ui);
    [LibraryImport(Library, EntryPoint = "cui_last_id")]
    internal static partial uint LastId(nint ui);
    [LibraryImport(Library, EntryPoint = "cui_make_id", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial uint MakeCanvasId(nint ui, string name);
    [LibraryImport(Library, EntryPoint = "cui_push_focus_scope")]
    internal static partial void PushFocusScope(nint ui, uint id, byte active);
    [LibraryImport(Library, EntryPoint = "cui_pop_focus_scope")]
    internal static partial void PopFocusScope(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_drawing_canvas", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DrawingCanvas(nint context, string id, int widthKind, float width, int heightKind, float height, CuiDrawingShape* shapes, int count, int enabled, out float x, out float y, out float w, out float h, int wheel, out float wheelX, out float wheelY, nint paint, nint user, int pointer, int focusable, int componentInput, int focus, int cursor);
}
