using Fpas.Ui.Gui.Cui.Interop.Developer;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_query_builder", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevQueryBuilder(nint ui, string id, NativeDevField* fields, int fieldCount, NativeDevRule* rules, ref int count, int capacity, int maxDepth, int readOnly, int widthKind, float width, int enabled, out NativeDevResult result);
}
