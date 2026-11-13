using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeEventOptions
{
    internal double Minimum, Maximum, Step, LabelStep;
    internal byte* Format;
    internal float Width;
    internal int WidthKind, Units, Flags;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_event_track", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void EventTrack(nint context, string id, double* times, ref int count, int capacity,
        in NativeEventOptions options, int* result);
}
