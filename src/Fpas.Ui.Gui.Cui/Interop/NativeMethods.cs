using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Ui.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    private const string Library = "fpas_cui";

    [LibraryImport(Library, EntryPoint = "fpas_cui_window_create", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint Create(string title, int width, int height, int renderer, int continuous);

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

    [LibraryImport(Library, EntryPoint = "cui_app_size")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiSize Size(nint app);

    [LibraryImport(Library, EntryPoint = "cui_set_reduced_motion")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ReducedMotion(nint context, byte enabled);

    [LibraryImport(Library, EntryPoint = "fpas_cui_column_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void BeginColumn(nint context, float padding, float gap);

    [LibraryImport(Library, EntryPoint = "fpas_cui_row_begin")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void BeginRow(nint context, float padding, float gap);

    [LibraryImport(Library, EntryPoint = "cui_box_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void EndBox(nint context);

    [LibraryImport(Library, EntryPoint = "fpas_cui_scroll_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void BeginScroll(nint context, string id, float height, int horizontal);

    [LibraryImport(Library, EntryPoint = "cui_scroll_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void EndScroll(nint context);

    [LibraryImport(Library, EntryPoint = "cui_scroll_to", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void ScrollTo(nint context, string id, float offset);

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
    internal static partial void LeftButton(nint context, int button, byte down);

    [LibraryImport(Library, EntryPoint = "fpas_cui_capture", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Capture(nint app, string path);
}
