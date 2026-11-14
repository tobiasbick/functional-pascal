using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_toast", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Toast(nint context, string title, string? description,
        int tone, float duration, string? action, int timer);

    [LibraryImport(Library, EntryPoint = "fpas_cui_toast_inspect")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ToastInspect(nint context, int handle, out int card, out int action, out int dismiss);
}
