using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativePresenceUser
{
    internal byte* Name, Avatar;
    internal int Palette;
    internal uint Rgba;
    internal int Idle;
}

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_presence", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void Presence(nint ui, string id, NativePresenceUser* users, int count, int maximum, float size);
}
