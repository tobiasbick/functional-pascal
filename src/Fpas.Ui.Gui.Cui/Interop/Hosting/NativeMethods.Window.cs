using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_app_minimize")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Minimize(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_restore")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Restore(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_window_state")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int WindowState(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_set_position")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void SetPosition(nint app, int x, int y);

    [LibraryImport(Library, EntryPoint = "fpas_cui_placement")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Placement(nint app, out int x, out int y, out int width, out int height, out int hasPosition);
}
