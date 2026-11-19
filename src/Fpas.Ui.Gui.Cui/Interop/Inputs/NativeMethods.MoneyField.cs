// [Documentation](../../../../docs/ui/gui/smart-inputs/money-field.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_money_field", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int MoneyField(nint context, string id, byte* text, nuint capacity,
        string? label, string symbol, string? code, string? hint, string? error,
        int widthKind, float width, int enabled, int focus, out int node);
}
