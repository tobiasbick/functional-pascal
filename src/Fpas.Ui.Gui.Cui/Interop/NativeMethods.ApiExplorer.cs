using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_api_explorer", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevApiExplorer(nint ui, string id, NativeDevEndpoint* endpoints, int endpoints_count, int selection, float height, int width_kind, float width, int enabled, out NativeDevResult result);
}
