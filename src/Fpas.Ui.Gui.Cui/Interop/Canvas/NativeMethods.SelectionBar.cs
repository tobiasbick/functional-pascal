using Fpas.Ui.Gui.Cui.Interop.Canvas;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_selection_bar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int SelectionBar(nint ui, string id, byte* selected, int count, NativeSelectionAction* actions, int actionCount, int enabled, int noEscape);
}
