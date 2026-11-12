using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_ruler_slider", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int RulerSlider(nint context, string id, string label, ref float value,
        float minimum, float maximum, float step, int majorEvery, float spacing, float width,
        string format, string? unit, int enabled, int focus, out int track);
}
