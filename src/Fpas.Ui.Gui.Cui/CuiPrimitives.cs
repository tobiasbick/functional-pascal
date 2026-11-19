// [Documentation](../../docs/ui/gui/cui.md#window-ownership-and-frames)
using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Selects the requested native rendering backend.</summary>
public enum CuiRenderer
{
    /// <summary>Chooses a renderer suitable for the current machine.</summary>
    Auto,
    /// <summary>Requests OpenGL, with CUI software fallback if unavailable.</summary>
    OpenGl,
    /// <summary>Uses CUI's software rasterizer.</summary>
    Software
}

/// <summary>A native widget rectangle in logical window units.</summary>
[StructLayout(LayoutKind.Sequential)]
public struct CuiRectangle
{
    /// <summary>The left edge.</summary>
    public float X;
    /// <summary>The top edge.</summary>
    public float Y;
    /// <summary>The width.</summary>
    public float Width;
    /// <summary>The height.</summary>
    public float Height;
}

/// <summary>A native window size in logical units.</summary>
[StructLayout(LayoutKind.Sequential)]
public struct CuiSize
{
    /// <summary>The logical width.</summary>
    public float Width;
    /// <summary>The logical height.</summary>
    public float Height;
}

/// <summary>How a native window is shown.</summary>
public enum CuiWindowState
{
    /// <summary>Neither maximized nor minimized.</summary>
    Normal,
    /// <summary>Maximized on its display.</summary>
    Maximized,
    /// <summary>Minimized to the taskbar or dock.</summary>
    Minimized
}

/// <summary>A window's normal size and place, and whether it is or comes back maximized.</summary>
/// <remarks>
/// Save it when the application closes and pass it to <see cref="CuiWindow"/> to reopen there.
/// The position is the content's top-left corner in desktop coordinates: pixels on Windows and
/// X11, points on macOS. Keep it only to pass it back. A place on no screen is left to the system.
/// </remarks>
public struct CuiWindowPlacement
{
    /// <summary>The left edge of the content in desktop coordinates.</summary>
    public int X;
    /// <summary>The top edge of the content in desktop coordinates, growing down.</summary>
    public int Y;
    /// <summary>The normal logical width.</summary>
    public int Width;
    /// <summary>The normal logical height.</summary>
    public int Height;
    /// <summary>Normal or maximized; a minimized window reports the state it comes back in.</summary>
    public CuiWindowState State;
    /// <summary>Whether X and Y name a place; false where the system reports none (Wayland).</summary>
    public bool HasPosition;
}
