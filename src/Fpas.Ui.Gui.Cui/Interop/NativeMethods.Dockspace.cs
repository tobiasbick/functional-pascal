using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_dockspace_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DockspaceBegin(nint ui, string id, string? layout, int shortcuts, int widthKind, float width, int heightKind, float height, float minimumWidth, float minimumHeight, int enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_dock_panel_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int DockPanelBegin(nint ui, string title, string? icon, int dirty, int closable, ref int open, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_dock_panel_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DockPanelEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_dockspace_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DockspaceEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_dockspace_load", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int DockspaceLoad(nint ui, string id, string layout);

    [LibraryImport(Library, EntryPoint = "fpas_cui_dockspace_save", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial nuint DockspaceSave(nint ui, string id, byte* buffer, nuint capacity);
}
