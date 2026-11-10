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

/// <summary>Reports native text-input events from one frame.</summary>
[Flags]
public enum CuiTextInputEvents
{
    /// <summary>No input event occurred.</summary>
    None = 0,
    /// <summary>The editing buffer changed.</summary>
    Changed = 1,
    /// <summary>The user submitted the field.</summary>
    Submitted = 2,
    /// <summary>The field received focus.</summary>
    Focused = 4,
    /// <summary>The field lost focus.</summary>
    Blurred = 8
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
