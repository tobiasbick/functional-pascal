using Fpas.Ui.Gui.Cui.Interop.Developer;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_dependency_graph", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevDependencyGraph(nint ui, string id, NativeDevGraphNode* vertices, int vertices_count, NativeDevEdge* edges, int edges_count, int selection, float height, int top_down, int width_kind, float width, int enabled, out NativeDevResult result);
}
