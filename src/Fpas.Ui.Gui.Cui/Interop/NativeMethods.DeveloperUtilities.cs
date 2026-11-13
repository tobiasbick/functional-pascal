using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_url_params", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int UrlParameters(string url, NativeDevPair* rows, int cap);

    [LibraryImport(Library, EntryPoint = "cui_url_set_params")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial byte SetUrlParameters(byte* url, nuint cap, NativeDevPair* rows, int count);

    [LibraryImport(Library, EntryPoint = "cui_query_text")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial nuint DescribeQuery(NativeDevField* fields, int fieldCount, NativeDevRule* rules, int count,
        byte* text, nuint capacity);
}
