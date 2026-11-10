using System.Runtime.InteropServices;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A frame-copied canvas rectangle or line in logical coordinates.</summary>
[StructLayout(LayoutKind.Sequential)]
public struct CuiPaint
{
    /// <summary>Zero for a rectangle, one for a line.</summary>
    public int Kind;
    /// <summary>Origin or first endpoint x.</summary>
    public float A;
    /// <summary>Origin or first endpoint y.</summary>
    public float B;
    /// <summary>Width or second endpoint x.</summary>
    public float C;
    /// <summary>Height or second endpoint y.</summary>
    public float D;
    /// <summary>Positive line thickness.</summary>
    public float Thickness;
    /// <summary>Red in the low byte, then green, blue and alpha.</summary>
    public uint Color;
}
