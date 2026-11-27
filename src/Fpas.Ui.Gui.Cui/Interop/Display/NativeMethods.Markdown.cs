// [Documentation](../../../../docs/ui/gui/code/markdown.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_markdown", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint Markdown(nint context, string id, string text, int widthKind, float width, float size, int flags, uint color, out int offset, out int line, out int check, out nint hovered, out int hoverChanged, out float height);
}
