using Fpas.Ui.Gui.Cui.Interop.Developer;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_network_waterfall", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevNetworkWaterfall(nint ui, string id, NativeDevRequest* requests, int requests_count, int selection, float height, double dom_loaded, double loaded, int width_kind, float width, int enabled, out NativeDevResult result);
}
