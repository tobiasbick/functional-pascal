using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_feature_flags", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevFeatureFlags(nint ui, string id, NativeDevFlag* flags, int count, byte** environments, int environmentCount, float height, int widthKind, float width, int enabled, out NativeDevResult result);
}
