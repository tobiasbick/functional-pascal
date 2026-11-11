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
