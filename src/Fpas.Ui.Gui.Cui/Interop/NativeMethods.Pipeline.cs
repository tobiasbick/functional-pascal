using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_pipeline", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevPipeline(nint ui, string id, byte** stages, int stages_count, NativeDevJob* jobs, int jobs_count, int selection, int width_kind, float width, int enabled, out NativeDevResult result);
}
