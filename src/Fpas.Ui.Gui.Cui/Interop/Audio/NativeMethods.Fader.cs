using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_fader", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Fader(nint context, string id, string label, ref float value,
        float minimum, float maximum, float step, float reset, string format, string? unit,
        int hideValue, float length, int horizontal, float* ticks, int tickCount, float* levels,
        int channels, int enabled, int focus, out int control);
}
