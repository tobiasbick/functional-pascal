using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_code_editor", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int CodeEditor(nint context, string id, byte* text, nuint capacity, string language, string? placeholder, int widthKind, float width, float height, float size, int tabSize, int flags, out int line, out int column, out int lines);
}
