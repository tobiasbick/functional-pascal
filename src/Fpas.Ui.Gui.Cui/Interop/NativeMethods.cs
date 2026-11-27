// [Documentation](../../../docs/ui/gui/cui.md#source-layout)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    private const string Library = "fpas_cui";

    [LibraryImport(Library, EntryPoint = "fpas_cui_window_create", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint Create(string title, int width, int height, int renderer, int continuous,
        int placed, int x, int y, int hasPosition, int maximized,
        int minimumWidth, int minimumHeight, int fixedSize, string? appId, int noVsync, int hidden,
        int readingDirection);

    [LibraryImport(Library, EntryPoint = "cui_app_destroy")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Destroy(nint app);

    [LibraryImport(Library, EntryPoint = "fpas_cui_finish_frame_for_destroy")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void FinishFrameForDestroy(nint app);

    [LibraryImport(Library, EntryPoint = "fpas_cui_frame")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Pump(nint app, int deferClose);

    [LibraryImport(Library, EntryPoint = "cui_app_quit")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Quit(nint app);

    [LibraryImport(Library, EntryPoint = "fpas_cui_present")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Present(nint app);

    [LibraryImport(Library, EntryPoint = "fpas_cui_last_node")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int LastNode(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_node_rect")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiRectangle NodeRectangle(nint context, int node);

    [LibraryImport(Library, EntryPoint = "cui_push_disabled")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void PushDisabled(nint context, byte disabled);

    [LibraryImport(Library, EntryPoint = "cui_pop_disabled")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void PopDisabled(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_disabled_scope")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void DisabledScope(nint context, int entering);

    [LibraryImport(Library, EntryPoint = "fpas_cui_wake_open")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint OpenWakeTarget(nint app);

    [LibraryImport(Library, EntryPoint = "fpas_cui_wake_post")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int PostWake(nint target);

    [LibraryImport(Library, EntryPoint = "fpas_cui_wake_close")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void CloseWakeTarget(nint target);

    [LibraryImport(Library, EntryPoint = "cui_app_ui")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint Context(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_backend")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint Backend(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_is_software")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte IsSoftware(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_set_size")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Resize(nint app, int width, int height);

    [LibraryImport(Library, EntryPoint = "cui_app_maximize")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Maximize(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_size")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiSize Size(nint app);

    [LibraryImport(Library, EntryPoint = "cui_viewport_size")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiSize ViewportSize(nint context);

    [LibraryImport(Library, EntryPoint = "cui_dpi_scale")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial float DpiScale(nint context);

    [LibraryImport(Library, EntryPoint = "cui_window_focused")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte WindowFocused(nint context);

    [LibraryImport(Library, EntryPoint = "cui_file_drag_active")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte FileDragActive(nint context);

    [LibraryImport(Library, EntryPoint = "cui_dropped_file_count")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int DroppedFileCount(nint context);

    [LibraryImport(Library, EntryPoint = "cui_set_reduced_motion")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ReducedMotion(nint context, byte enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_column_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int BeginColumn(nint context, float padding, float gap);

    [LibraryImport(Library, EntryPoint = "fpas_cui_row_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int BeginRow(nint context, float padding, float gap);

    [LibraryImport(Library, EntryPoint = "fpas_cui_scroll_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int BeginScroll(nint context, string id, float height, int horizontal);

    [LibraryImport(Library, EntryPoint = "cui_scroll_to", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ScrollTo(nint context, string id, float offset);

    [LibraryImport(Library, EntryPoint = "cui_scroll_reveal", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ScrollReveal(nint context, string id, uint row);

    [LibraryImport(Library, EntryPoint = "fpas_cui_request_focus", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void RequestFocus(nint context, string id);

    [LibraryImport(Library, EntryPoint = "cui_heading", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Heading(nint context, string text, int level);

    [LibraryImport(Library, EntryPoint = "cui_text_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Text(nint context, string text, nint options);

    [LibraryImport(Library, EntryPoint = "cui_last_rect")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiRectangle LastRectangle(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_move_pointer")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void MovePointer(nint context, float x, float y);

    [LibraryImport(Library, EntryPoint = "cui_input_mouse_button")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void MouseButton(nint context, int button, byte down);

    [LibraryImport(Library, EntryPoint = "fpas_cui_capture", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Capture(nint app, string path);
}
