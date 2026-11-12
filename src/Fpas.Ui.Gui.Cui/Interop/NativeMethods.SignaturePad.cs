using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_signature_pad", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int SignaturePad(nint context, string id, string? placeholder, float height, uint rgba,
        int enabled, int focusClear, out int node, out int clear);

    [LibraryImport(Library, EntryPoint = "cui_signature_points", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int SignaturePoints(nint context, string id, out nint points,
        out nint strokeEnds, out int strokeCount);
}
