// [Documentation](../../../../docs/ui/gui/selection/slider.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_slider", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Slider(nint context, string id, string label, ref float value,
        float minimum, float maximum, float step, int showValue, string? format, int widthKind, float width,
        int enabled, int focus, out int track);
}
