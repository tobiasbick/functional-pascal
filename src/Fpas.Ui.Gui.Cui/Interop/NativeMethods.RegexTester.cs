using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_regex_tester", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void DevRegexTester(nint ui, string id, byte* pattern, nuint patternCapacity, byte* text, nuint textCapacity, int modifiers, float height, int widthKind, float width, int enabled, out NativeDevResult result);
}
