// [Documentation](../../../../docs/ui/gui/cui.md#touch-input)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_input_touch")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void InputTouch(nint context, int finger, byte down, float x, float y);
    [LibraryImport(Library, EntryPoint = "cui_input_touch_move")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void InputTouchMove(nint context, int finger, float x, float y);
    [LibraryImport(Library, EntryPoint = "cui_input_touch_cancel")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void InputTouchCancel(nint context, int finger);
    [LibraryImport(Library, EntryPoint = "cui_touch_count")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TouchCount(nint context);
    [LibraryImport(Library, EntryPoint = "cui_touch_down")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte TouchDown(nint context, int finger);
    [LibraryImport(Library, EntryPoint = "cui_touch_pressed")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte TouchPressed(nint context, int finger);
    [LibraryImport(Library, EntryPoint = "cui_touch_released")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte TouchReleased(nint context, int finger);
    [LibraryImport(Library, EntryPoint = "cui_touch_pos")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiPoint TouchPosition(nint context, int finger);
}
