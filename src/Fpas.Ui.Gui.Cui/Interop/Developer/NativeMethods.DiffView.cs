// [Documentation](../../../../docs/ui/gui/code/diff-viewer.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Widgets.Developer;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_diff_view", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int DiffView(nint context, string id, string before, string after, int current, string language, string? oldTitle, string? newTitle, int contextLines, int widthKind, float width, float height, float size, float splitMinWidth, int flags, out NativeDiffObservation observation);

    [LibraryImport(Library, EntryPoint = "fpas_cui_diff_view_rows", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int DiffViewRows(nint context, string id, CuiDiffRow* rows, int first, int capacity);
}

[StructLayout(LayoutKind.Sequential)]
internal struct NativeDiffObservation
{
    internal int Current, Count, Inline, Added, Removed, Folds, Hidden, Approximate, CurrentChanged, Recomputed;
}
