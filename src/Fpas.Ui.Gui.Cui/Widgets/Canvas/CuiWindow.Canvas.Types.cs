using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>A primitive painted in local canvas points.</summary>
/// <param name="Kind">Zero for rectangle, one for circle or two for line.</param>
/// <param name="X">Horizontal origin.</param>
/// <param name="Y">Vertical origin.</param>
/// <param name="A">Rectangle width or line endpoint X.</param>
/// <param name="B">Rectangle height or line endpoint Y.</param>
/// <param name="Radius">Corner radius, circle radius, or line thickness.</param>
/// <param name="Color">Packed straight-alpha RGBA.</param>
[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
public readonly record struct CuiDrawingShape(int Kind, float X, float Y, float A, float B, float Radius, uint Color);
