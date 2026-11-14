using Fpas.Ui.Gui.Cui.Widgets.Editors;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_property_inspector", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void PropertyInspector(nint context, string id, NativePropertyGroup* groups, int count, float labelWidth, int widthKind, float width, int enabled);
}
