// [Documentation](../../../../docs/ui/gui/overlays/guided-tour.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_tour", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Tour(nint context, string id, uint* targets,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] titles,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] bodies,
        int* placements, int count, ref int step, float width, float padding, int blockTarget,
        string? nextLabel, string? backLabel, string? doneLabel, string? countFormat,
        int focus, out int node, out uint key, int* parts);
}
