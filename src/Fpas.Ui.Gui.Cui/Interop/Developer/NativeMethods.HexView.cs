using Fpas.Ui.Gui.Cui.Widgets.Developer;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_hex_view", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int HexView(nint context, string id, byte* bytes, int count, ref int start, ref int length, ref int cursor, NativeHexMark* marks, int markCount, ulong address, int goTo, int bytesPerRow, int widthKind, float width, float height, float size, int flags, out int offset, out int editLength);
}
