// [Documentation](../../../docs/ui/gui/app-chrome/window.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Gets the borrowed, backend-specific handle on the owner thread until the window is disposed.</summary>
    public nint NativeWindow { get { CheckOpen(); return NativeMethods.NativeWindow(app); } }

    /// <summary>Gets whether the window is normal, maximized or minimized, as the system reports it now.</summary>
    /// <remarks>On X11 the window manager carries out state requests, so a change can take a few frames to show.</remarks>
    public CuiWindowState State { get { CheckOpen(); return (CuiWindowState)NativeMethods.WindowState(app); } }

    /// <summary>Gets the normal size and place, also while the window is maximized or minimized.</summary>
    /// <remarks>
    /// The state is <see cref="CuiWindowState.Maximized"/> when the window is or comes back maximized,
    /// so saving it while minimized from maximized reopens a maximized window.
    /// </remarks>
    public CuiWindowPlacement Placement
    {
        get
        {
            CheckOpen();
            var state = (CuiWindowState)NativeMethods.Placement(app, out int x, out int y, out int width, out int height, out int hasPosition);
            return new CuiWindowPlacement { X = x, Y = y, Width = width, Height = height, State = state, HasPosition = hasPosition != 0 };
        }
    }

    /// <summary>Minimizes the window where the window manager supports it.</summary>
    public void Minimize()
    {
        CheckOpen();
        NativeMethods.Minimize(app);
    }

    /// <summary>Undoes a maximize or a minimize; a window minimized from maximized comes back maximized.</summary>
    public void Restore()
    {
        CheckOpen();
        NativeMethods.Restore(app);
    }

    /// <summary>Moves the top-left corner of the content to a place in desktop coordinates.</summary>
    /// <param name="x">Left edge: pixels on Windows and X11, points on macOS.</param>
    /// <param name="y">Top edge, growing down, in the same units.</param>
    /// <remarks>Wayland ignores the request because it lets no window place itself.</remarks>
    public void MoveTo(int x, int y)
    {
        CheckOpen();
        NativeMethods.SetPosition(app, x, y);
    }
}
