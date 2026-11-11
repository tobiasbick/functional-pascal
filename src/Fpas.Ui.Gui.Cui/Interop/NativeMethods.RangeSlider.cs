using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_range_slider", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int RangeSlider(nint context, string id, string label, ref float lower,
        ref float upper, float minimum, float maximum, int enabled, int focus, out int track);
}
