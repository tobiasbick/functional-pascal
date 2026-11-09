using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.Marshalling;

namespace Fpas.Gui.Cui.Interop;

internal static partial class NativeMethods
{
    private const string Library = "fpas_cui";

    [LibraryImport(Library, EntryPoint = "fpas_cui_window_create", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial nint Create(string title, int width, int height, int renderer, int continuous);

    [LibraryImport(Library, EntryPoint = "cui_app_destroy")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Destroy(nint app);

    [LibraryImport(Library, EntryPoint = "cui_app_frame")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte Frame(nint app);

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

    [LibraryImport(Library, EntryPoint = "fpas_cui_request_focus", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void RequestFocus(nint context, string id);

    [LibraryImport(Library, EntryPoint = "cui_heading", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Heading(nint context, string text, int level);

    [LibraryImport(Library, EntryPoint = "cui_text_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Text(nint context, string text, nint options);

    [LibraryImport(Library, EntryPoint = "cui_button_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte Button(nint context, string label, nint options);

    [LibraryImport(Library, EntryPoint = "fpas_cui_list_item", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int ListItem(nint context, string label, int selected);

    [LibraryImport(Library, EntryPoint = "fpas_cui_tab_item", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TabItem(nint context, string label, int selected);

    [LibraryImport(Library, EntryPoint = "fpas_cui_table_begin", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int BeginTable(nint context, string id,
        [MarshalUsing(typeof(Utf8StringMarshaller), ElementIndirectionDepth = 1)] string[] titles, int count);

    [LibraryImport(Library, EntryPoint = "fpas_cui_table_row")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int TableRow(nint context, int selected);

    [LibraryImport(Library, EntryPoint = "cui_table_cell_text", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void TableCell(nint context, string text);

    [LibraryImport(Library, EntryPoint = "cui_table_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void EndTable(nint context);

    [LibraryImport(Library, EntryPoint = "cui_checkbox_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte Checkbox(nint context, string label, ref byte value, nint options);

    [LibraryImport(Library, EntryPoint = "fpas_cui_text_input", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Input(nint context, string id, byte* buffer, nuint capacity);

    [LibraryImport(Library, EntryPoint = "cui_last_rect")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial CuiRectangle LastRectangle(nint context);

    [LibraryImport(Library, EntryPoint = "cui_input_mouse_move")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void MovePointer(nint context, float x, float y);

    [LibraryImport(Library, EntryPoint = "cui_input_mouse_button")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void LeftButton(nint context, int button, byte down);

    [LibraryImport(Library, EntryPoint = "cui_input_text", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void TypeText(nint context, string text);

    [LibraryImport(Library, EntryPoint = "fpas_cui_capture", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int Capture(nint app, string path);
}
