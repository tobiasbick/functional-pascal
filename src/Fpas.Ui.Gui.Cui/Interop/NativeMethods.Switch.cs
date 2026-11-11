using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_switch", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Switch(nint context, string id, string label, string? description,
        int on, int enabled, int focus);
}
