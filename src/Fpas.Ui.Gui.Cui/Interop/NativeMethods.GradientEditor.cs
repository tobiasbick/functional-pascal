using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_gradient_editor", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int GradientEditor(nint context, string id, CuiGradientStop* stops, ref int count, int capacity, ref int selected, out int added, out int removed, int widthKind, float width, float height, int enabled);
}
