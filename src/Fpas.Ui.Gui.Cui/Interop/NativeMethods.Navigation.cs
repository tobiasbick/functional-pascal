using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_sidebar_clear", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ClearSidebar(nint context, string group);

    [LibraryImport(Library, EntryPoint = "cui_nav_section", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void SidebarHeading(nint context, string title);

    [LibraryImport(Library, EntryPoint = "fpas_cui_sidebar_item", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int SidebarItem(nint context, string group, string id, string text,
        string? icon, string? badge, int enabled, int selected, int focus);
}
