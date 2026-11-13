using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_voice_resample")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ResampleVoice(float* levels, int count, float* output, int columns);

    [LibraryImport(Library, EntryPoint = "fpas_cui_voice_note", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int VoiceNote(nint ui, string? id, float* levels, int count, float duration, float* position, float* speed, int* playing, int width_kind, float width, int flags, int focus, int* nodes);
}
