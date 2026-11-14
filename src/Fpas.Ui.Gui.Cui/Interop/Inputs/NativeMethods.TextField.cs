using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_text_field", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int TextField(nint context, string id, byte* value, nuint capacity,
        string? label, string? placeholder, string? icon, string? hint, string? error, string? prefix, string? suffix,
        int size, int password, int clearable, int readOnly, int enabled, int focus, int* nodes, out int revealed);

    [LibraryImport(Library, EntryPoint = "cui_input_text", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void InputText(nint context, string text);
}
