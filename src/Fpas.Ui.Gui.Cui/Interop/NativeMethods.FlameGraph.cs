using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_flame_graph", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int FlameGraph(nint context, string id, NativeFlameSpan* spans, int count, ref NativeFlameRange view, double minimum, float height, float rowHeight, int widthKind, float width, int flags);
}
