using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

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

    [LibraryImport(Library, EntryPoint = "cui_box_end")]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void EndColumn(nint context);

    [LibraryImport(Library, EntryPoint = "cui_heading", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Heading(nint context, string text, int level);

    [LibraryImport(Library, EntryPoint = "cui_text_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void Text(nint context, string text, nint options);

    [LibraryImport(Library, EntryPoint = "cui_button_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte Button(nint context, string label, nint options);

    [LibraryImport(Library, EntryPoint = "cui_checkbox_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial byte Checkbox(nint context, string label, ref byte value, nint options);

    [LibraryImport(Library, EntryPoint = "cui_input_ex", StringMarshalling = StringMarshalling.Utf8)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static unsafe partial int Input(nint context, string id, byte* buffer, nuint capacity, nint options);

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
