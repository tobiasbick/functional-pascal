using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_scrub_field", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ScrubField(nint context, string id, string label, ref float value,
        float minimum, float maximum, float step, string format, string? suffix, string? icon,
        int widthKind, float width, int enabled, int focus, out int node, out uint key);
}
