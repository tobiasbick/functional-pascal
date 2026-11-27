// [Documentation](../../../docs/ui/gui/app-chrome/window.md)
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

public sealed class CuiWindowObservationTests
{
    [GuiFact]
    public void ObservationsFollowFocusDragAndUnclaimedDrops()
    {
        using var window = new CuiWindow("Window observation", continuous: true, renderer: CuiRenderer.Software);
        nint context = Context(window);
        Assert.True(float.IsFinite(window.DpiScale) && window.DpiScale > 0);
        Assert.Equal(0, window.DroppedFileCount);
        Assert.True(window.NextFrame());
        InputFocus(context, 0);
        Assert.False(window.IsFocused);
        InputFocus(context, 1);
        Assert.True(window.IsFocused);
        InputFileHover(context, 1, 100, 100);
        Assert.True(window.FileDragActive);
        InputFileHover(context, 0, 100, 100);
        Assert.False(window.FileDragActive);
        Drop(context, 100, 100);
        Assert.Equal(2, window.DroppedFileCount);
        window.Text("Unclaimed");
        window.Present();
        Assert.Equal(2, window.DroppedFileCount);
        Assert.True(window.NextFrame());
        Assert.Equal(0, window.DroppedFileCount);
        window.DiscardFrame();
        Assert.Equal(2, window.DroppedFileCount);

        Assert.True(window.NextFrame());
        DrawDropzone();
        window.Present();
        var bounds = window.GetControlRectangle("drop");
        Assert.True(bounds.Width > 0 && bounds.Height > 0);
        Assert.True(window.NextFrame());
        Drop(context, bounds.X + bounds.Width / 2, bounds.Y + bounds.Height / 2);
        Assert.Equal(2, window.DroppedFileCount);
        Assert.Equal(2, DrawDropzone());
        Assert.Equal(0, window.DroppedFileCount);
        window.Present();
        Assert.Equal(0, window.DroppedFileCount);

        int DrawDropzone()
        {
            window.BeginColumn();
            var result = window.Dropzone("drop", "Drop", null, "", null, 100, false, true, false, true);
            window.EndColumn();
            return result.Paths.Length;
        }
    }

    [UnsafeAccessor(UnsafeAccessorKind.Field, Name = "context")]
    private static extern ref nint Context(CuiWindow window);

    [DllImport("fpas_cui", EntryPoint = "cui_input_focus", CallingConvention = CallingConvention.Cdecl)]
    private static extern void InputFocus(nint context, byte focused);

    [DllImport("fpas_cui", EntryPoint = "cui_input_file_hover", CallingConvention = CallingConvention.Cdecl)]
    private static extern void InputFileHover(nint context, byte hovering, float x, float y);

    [DllImport("fpas_cui", EntryPoint = "cui_input_file_drop", CallingConvention = CallingConvention.Cdecl)]
    private static extern unsafe void InputFileDrop(nint context, nint* paths, int count, float x, float y);

    private static unsafe void Drop(nint context, float x, float y)
    {
        fixed (byte* first = "first.txt\0"u8, second = "second.txt\0"u8)
        {
            nint* paths = stackalloc nint[] { (nint)first, (nint)second };
            InputFileDrop(context, paths, 2, x, y);
        }
    }
}
