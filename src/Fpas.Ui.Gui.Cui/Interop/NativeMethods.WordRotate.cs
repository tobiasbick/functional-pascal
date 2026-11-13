using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_word_rotate", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void WordRotate(nint context, string id, [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] words, int count, float interval, in NativeMotionText options);
}
