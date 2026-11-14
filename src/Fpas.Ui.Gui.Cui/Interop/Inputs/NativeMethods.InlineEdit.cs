using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_inline_edit", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int InlineEdit(nint context, string id, byte* value, nuint capacity,
        string? placeholder, int font, float size, int widthKind, float width, int enabled, int focus,
        out int editing, out int node, out uint key);
}
