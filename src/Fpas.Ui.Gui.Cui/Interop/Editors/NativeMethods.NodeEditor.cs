using Fpas.Ui.Gui.Cui.Widgets.Editors;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "fpas_cui_node_editor_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int NodeEditorBegin(nint ui, string id, CuiNodeView view, int widthKind, float width, int heightKind, float height, float minimum, float maximum, int flags);

    [LibraryImport(Library, EntryPoint = "fpas_cui_node_editor_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void NodeEditorEnd(nint ui, out CuiNodeView result);

    [LibraryImport(Library, EntryPoint = "fpas_cui_node_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void NodeBegin(nint ui, int id, ref float x, ref float y, string title, float width, string? icon);

    [LibraryImport(Library, EntryPoint = "fpas_cui_node_pin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void NodePin(nint ui, int id, string label, int output, int type);

    [LibraryImport(Library, EntryPoint = "cui_node_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void NodeEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "cui_node_content_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void NodeContentBegin(nint ui);

    [LibraryImport(Library, EntryPoint = "cui_node_content_end", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void NodeContentEnd(nint ui);

    [LibraryImport(Library, EntryPoint = "cui_node_link", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial byte NodeLink(nint ui, int id, int fromPin, int toPin);

    [LibraryImport(Library, EntryPoint = "fpas_cui_node_event", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int NodeEvent(nint ui, int kind, out int first, out int second);
}
