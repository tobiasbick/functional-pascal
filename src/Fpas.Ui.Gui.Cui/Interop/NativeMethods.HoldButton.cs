using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_hold_button", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int HoldButton(nint context, string id, string label, string? icon,
        int durationMilliseconds, int variant, int enabled, int focus);
}
