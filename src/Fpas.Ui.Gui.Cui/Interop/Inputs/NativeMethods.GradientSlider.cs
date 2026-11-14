using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_gradient_slider", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int GradientSlider(nint context, string id, string label, ref float value, float minimum, float maximum, int track, float* positions, uint* colors, int count, int palette, uint rgba, float step, byte* format, string? display, string? tool, string? tooltip, int flags, int focus, out uint sample, int* parts);
}
