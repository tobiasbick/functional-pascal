using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_status_bar_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void StatusBarBegin(nint ui, string id, int border);

    [LibraryImport(Library, EntryPoint = "fpas_cui_status_bar_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void StatusBarEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_status_bar_spacer", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void StatusBarSpacer(nint ui);

    [LibraryImport(Library, EntryPoint = "fpas_cui_status_bar_text", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void StatusBarText(nint ui, string text, string? icon, int tone);

    [LibraryImport(Library, EntryPoint = "fpas_cui_status_bar_item", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int StatusBarItem(nint ui, string title, string? icon, int tone, string? tooltip, int enabled, int focus, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_status_bar_progress", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void StatusBarProgress(nint ui, float value, string? label, float width, int indeterminate);
}
