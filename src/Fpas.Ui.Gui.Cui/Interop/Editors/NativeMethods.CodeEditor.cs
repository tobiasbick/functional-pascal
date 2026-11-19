// [Documentation](../../../../docs/ui/gui/code/code-editor.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_code_editor", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int CodeEditor(nint context, string id, byte* text, nuint capacity, string language, string? placeholder, int widthKind, float width, float height, float size, int tabSize, int flags, NativeTextMark* marks, int markCount, out NativeCodeEditor result);

    [LibraryImport(Library, EntryPoint = "cui_code_editor_goto", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CodeEditorGoTo(nint context, string id, int line, int column);

    [LibraryImport(Library, EntryPoint = "cui_code_editor_select", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CodeEditorSelect(nint context, string id, int anchor, int cursor);

    [LibraryImport(Library, EntryPoint = "cui_code_editor_find", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CodeEditorFind(nint context, string id, string? query);
}
