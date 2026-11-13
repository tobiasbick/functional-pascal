using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_diff_view", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DiffView(nint context, string id, string before, string after, int current, string language, string? oldTitle, string? newTitle, int contextLines, int widthKind, float width, float height, float size, int flags, out int selected, out int count, out int inlineMode);
}
