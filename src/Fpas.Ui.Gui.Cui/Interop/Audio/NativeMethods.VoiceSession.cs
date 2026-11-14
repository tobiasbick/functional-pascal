using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_voice_session", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int VoiceSession(nint ui, string? id, int activity, float* levels, byte* owners, int count, string? label, string? transcript, string? upcoming, int speaker, string? you, string? assistant, int* muted, int width_kind, float width, int flags, int focus, int* nodes);
}
