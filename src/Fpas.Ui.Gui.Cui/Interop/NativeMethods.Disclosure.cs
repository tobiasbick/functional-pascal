using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_disclosure_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int DisclosureBegin(nint context, string id, string title, int kind,
        int open, int enabled, int focus, string description, string icon, string badge, int tone,
        out int node, out int header);

    [LibraryImport(Library, EntryPoint = "fpas_cui_disclosure_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DisclosureEnd(nint context, int kind, int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_expandable_text", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ExpandableText(nint context, string id, string text, int lines,
        float size, int colors, uint color, uint fadeTo, int enabled, int focus, out int toggle);
}
