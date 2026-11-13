using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_keymap", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevKeymap(nint ui, string id, NativeDevBinding* bindings, int bindings_count, byte** categories, int categories_count, int modifiers, int hide_function_row, int width_kind, float width, int enabled, out NativeDevResult result);
}
