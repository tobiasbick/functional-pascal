using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_money_field", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void MoneyField(nint context, string id, ref double value,
        string? label, string symbol, string? code, string? hint, int decimals,
        int widthKind, float width, int enabled, int focus, out int node);
}
