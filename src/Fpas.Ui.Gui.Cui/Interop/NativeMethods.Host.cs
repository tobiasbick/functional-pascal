using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    [LibraryImport(Library, EntryPoint = "cui_separator")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Separator(nint context);
    [LibraryImport(Library, EntryPoint = "fpas_cui_modal_key")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint ModalKey(nint context);
    [LibraryImport(Library, EntryPoint = "fpas_cui_viewport", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Viewport(nint context, string id, int scrollbar, int enabled);
    [LibraryImport(Library, EntryPoint = "fpas_cui_scroll_metrics")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ScrollMetrics(nint context, int node, out float offset, out float maximum);
    [LibraryImport(Library, EntryPoint = "fpas_cui_scroll_layout")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ScrollLayout(nint context, int node, int mode, float offset);
    [LibraryImport(Library, EntryPoint = "fpas_cui_resolve_layout")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ResolveLayout(nint context);
    [LibraryImport(Library, EntryPoint = "fpas_cui_set_resize_renderer")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ResizeRenderer(nint app, delegate* unmanaged[Cdecl]<nint, nint, void> callback, nint user);
    [LibraryImport(Library, EntryPoint = "fpas_cui_focus_key")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int FocusKey(nint context, uint key);
    [LibraryImport(Library, EntryPoint = "fpas_cui_host_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void HostBegin(nint context);
    [LibraryImport(Library, EntryPoint = "fpas_cui_node_key")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial uint NodeKey(nint context, int node);
    [LibraryImport(Library, EntryPoint = "fpas_cui_forget")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Forget(nint context, uint key);
    [LibraryImport(Library, EntryPoint = "fpas_cui_group_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int GroupBegin(nint context, int horizontal, float padding, float gap, int align, int framed, int wrap, int clip);
    [LibraryImport(Library, EntryPoint = "fpas_cui_group_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void GroupEnd(nint context, int node);
    [LibraryImport(Library, EntryPoint = "fpas_cui_node_size")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void NodeSize(nint context, int node, int horizontal, int kind, float basis, float minimum, float maximum, float grow, int align);
    [LibraryImport(Library, EntryPoint = "fpas_cui_node_role")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void NodeRole(nint context, int node, int role);
    [LibraryImport(Library, EntryPoint = "fpas_cui_focused")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Focused(nint context, int node);
    [LibraryImport(Library, EntryPoint = "fpas_cui_hovered")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Hovered(nint context, int node);
    [LibraryImport(Library, EntryPoint = "fpas_cui_scroll_state", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ScrollState(nint context, string id, int mode, float offset, out float current, out float maximum);
    [LibraryImport(Library, EntryPoint = "fpas_cui_theme")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void SetTheme(nint context, int dark, int accent, uint* colors);
    [LibraryImport(Library, EntryPoint = "fpas_cui_theme_read")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial void ReadTheme(nint context, uint* colors);
    [LibraryImport(Library, EntryPoint = "cui_key_pressed")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte KeyPressed(nint context, int key);
    [LibraryImport(Library, EntryPoint = "cui_key_mods")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int KeyModifiers(nint context);
    [LibraryImport(Library, EntryPoint = "cui_input_key")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void InputKey(nint context, int key, byte down, int modifiers);
    [LibraryImport(Library, EntryPoint = "cui_input_scroll")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void InputScroll(nint context, float x, float y);
}
