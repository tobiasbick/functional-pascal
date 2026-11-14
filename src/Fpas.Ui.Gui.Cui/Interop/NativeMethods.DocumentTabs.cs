using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_document_tabs", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DocumentTabs(nint ui, string id, string[] titles, string[] icons, string[] tips, int* dirty, int count, ref int selected, int flags, out int close, out int from, out int to, int* nodes);
}
