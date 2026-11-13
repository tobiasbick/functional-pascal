using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_key_value_editor", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevKeyValueEditor(nint ui, string id, NativeDevPair* rows, ref int count, int capacity, string? keyPlaceholder, string? valuePlaceholder, int hideToggles, int hideSecrets, int readOnly, int widthKind, float width, int enabled, out NativeDevResult result);
}
