using Fpas.Ui.Gui.Cui.Interop.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_format_bar", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int FormatBar(nint ui, string id, ref CuiTextFormat format, NativeTextStyle* styles, int styleCount, int* colors, int colorCount, string? footer, string? keys, string? question, int flags);
}
