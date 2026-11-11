using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_checkbox", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Checkbox(nint context, string id, string label, string? description,
        int @checked, int enabled, int focus, int indeterminate);
}
