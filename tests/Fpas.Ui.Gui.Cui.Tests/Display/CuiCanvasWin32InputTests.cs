// [Documentation](../../../docs/ui/gui/text-and-display/canvas.md)
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks that Win32 keyboard messages reach a focused canvas through CUI's input hook.</summary>
public sealed partial class CuiCanvasWin32InputTests
{
    private const uint KeyDown = 0x0100, KeyUp = 0x0101;

    /// <summary>Copies key state, text and release from the native event the window procedure creates.</summary>
    [WindowsGuiFact]
    public void PumpsWin32InputIntoFocusedCanvas()
    {
        using var window = new CuiWindow("Canvas Win32 input", 400, 240, CuiRenderer.Software, continuous: true);
        var events = new List<CuiCanvasInput>();
        Frame(true);
        Frame(false);
        events.Clear();
        nint handle = window.NativeWindow;
        // The window procedure's TranslateMessage posts the WM_CHAR for the key down.
        Assert.NotEqual(0, PostMessage(handle, KeyDown, 0x41, 0x001E0001));
        for (int frame = 0; frame < 60 && !events.Any(e => e.Kind == CuiCanvasInputKind.Text); frame++)
            Frame(false);
        Assert.NotEqual(0, PostMessage(handle, KeyUp, 0x41, unchecked((nint)0xC01E0001)));
        for (int frame = 0; frame < 60 && !events.Any(e => e.Kind == CuiCanvasInputKind.Key && !e.Down); frame++)
            Frame(false);
        Assert.Collection(events.Where(e => e.Kind != CuiCanvasInputKind.WindowFocus),
            down => Assert.True(down.Kind == CuiCanvasInputKind.Key && down.Down && !down.Repeated),
            text => Assert.Equal("a", text.Text),
            up => Assert.True(up.Kind == CuiCanvasInputKind.Key && !up.Down));

        void Frame(bool request)
        {
            Assert.True(window.NextFrame());
            window.DrawingCanvas("input", 1, 0, 2, 40, [], true, out _, out _, out _, out _, false, out _, out _,
                focus: request, onInput: events.Add);
            window.Present();
        }
    }

    [LibraryImport("user32", EntryPoint = "PostMessageW")]
    private static partial int PostMessage(nint window, uint message, nuint wParam, nint lParam);
}
