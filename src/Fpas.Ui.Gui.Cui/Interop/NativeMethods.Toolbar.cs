using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_toolbar_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ToolbarBegin(nint ui, string id, int style, int noBorder, int enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_toolbar_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ToolbarEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_toolbar_button", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ToolbarButton(nint ui, string? icon, string? label, string? tooltip, string? shortcut, int toggle, int tone, int flags);

    [LibraryImport(Library, EntryPoint = "fpas_cui_toolbar_radio", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ToolbarRadio(nint ui, string id, byte** icons, byte** labels, int count, int value);

    [LibraryImport(Library, EntryPoint = "fpas_cui_toolbar_cycle", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ToolbarCycle(nint ui, string id, byte** labels, int count, int value, string? tooltip);

    [LibraryImport(Library, EntryPoint = "cui_toolbar_group_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ToolbarGroupBegin(nint ui);

    [LibraryImport(Library, EntryPoint = "cui_toolbar_group_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ToolbarGroupEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "cui_toolbar_separator", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ToolbarSeparator(nint ui);

    [LibraryImport(Library, EntryPoint = "cui_toolbar_spacer", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ToolbarSpacer(nint ui);
}
