using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_banner", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Banner(nint context, string id, string text, int shown, string? action,
        string? icon, int tone, int dismissible, int enabled, int focus, out int node,
        out int actionNode, out int dismissNode, out uint key);
}
