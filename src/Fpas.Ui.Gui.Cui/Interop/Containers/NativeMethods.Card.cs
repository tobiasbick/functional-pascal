using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_card_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int CardBegin(nint context, string id, int widthKind, float width,
        int heightKind, float height, float minimum, float maximum, float padding, float paddingX,
        float paddingY, float gap, int row, int align, int wrap, int muted, int selected,
        int hoverable, int enabled, int focus, out int node);

    [LibraryImport(Library, EntryPoint = "fpas_cui_card_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CardEnd(nint context, int node);
}
