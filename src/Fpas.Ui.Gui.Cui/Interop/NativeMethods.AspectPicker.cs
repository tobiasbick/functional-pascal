using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_aspect_picker", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int AspectPicker(nint context, string id, string label, ref int value, int* widths, int* heights, string[] labels, int count, string? custom, int sizeBase, string? display, int flags, int focus, int* parts);
}
