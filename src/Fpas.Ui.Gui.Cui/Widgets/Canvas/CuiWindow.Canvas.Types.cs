// [Documentation](../../../../docs/ui/gui/text-and-display/canvas.md)
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

/// <summary>A native activation position in the preceding canvas layout.</summary>
/// <param name="X">Horizontal coordinate in canvas-local logical points.</param>
/// <param name="Y">Vertical coordinate in canvas-local logical points.</param>
/// <param name="Width">Canvas width in logical points.</param>
/// <param name="Height">Canvas height in logical points.</param>
public readonly record struct CuiCanvasClick(float X, float Y, float Width, float Height);

/// <summary>Events delivered to a canvas with component focus.</summary>
public enum CuiCanvasInputKind
{
    /// <summary>A physical key transition.</summary>
    Key = 4,
    /// <summary>Committed Unicode text.</summary>
    Text,
    /// <summary>The window gained or lost keyboard focus.</summary>
    WindowFocus
}

/// <summary>One copied native input event in delivery order.</summary>
/// <param name="Kind">Key, text or window focus.</param><param name="Key">Native key ordinal.</param>
/// <param name="Modifiers">Native modifier mask.</param><param name="Down">Key down or window focused.</param>
/// <param name="Repeated">Whether a key-down repeats.</param><param name="Text">Committed text, otherwise empty.</param>
public readonly record struct CuiCanvasInput(CuiCanvasInputKind Kind, int Key, int Modifiers, bool Down, bool Repeated, string Text);
