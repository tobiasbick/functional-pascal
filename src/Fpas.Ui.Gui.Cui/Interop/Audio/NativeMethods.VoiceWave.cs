using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_voice_wave", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int VoiceWave(nint ui, string? id, float* levels, byte* owners, int count, float* progress, int width_kind, float width, float pitch, float step, int owner, int palette, uint rgba, int flags);
}
