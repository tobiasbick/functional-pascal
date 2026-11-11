using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_action_footer_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ActionFooterBegin(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_action_footer_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ActionFooterEnd(nint context, int footer);
}
