// [Documentation](../../../../docs/ui/gui/canvas-and-bars/result-card.md)
using Fpas.Ui.Gui.Cui.Interop.Media;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_result_card", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int ResultCard(nint ui, string id, NativeMediaSource* source, string title,
        string? caption, string? badge, int tone, int widthKind, float width, float aspect, int enabled, int focus,
        int stretchImage);
}
