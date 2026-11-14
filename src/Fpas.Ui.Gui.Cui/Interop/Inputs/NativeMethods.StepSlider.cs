using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_step_slider", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int StepSlider(nint context, string id, string label, ref int value, int minimum, int maximum, string[] labels, int count, string? format, string? display, string? tool, string? tooltip, int flags, int focus, int* parts);
}
