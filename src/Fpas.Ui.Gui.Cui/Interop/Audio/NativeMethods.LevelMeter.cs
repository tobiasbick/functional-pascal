using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_level_meter", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int LevelMeter(nint context, string id, float* levels, int channels,
        float minimumDb, float maximumDb, float length, float thickness, int horizontal,
        int scale, int segmented, float hold, float decay, int enabled, out int control);
}
