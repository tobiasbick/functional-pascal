using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Media;

/// <summary>Image zoom and center in pixels.</summary>
/// <param name="Zoom">Points per pixel, or zero to fit.</param>
/// <param name="CenterX">Horizontal center in image pixels.</param>
/// <param name="CenterY">Vertical center in image pixels.</param>
[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
public readonly record struct CuiImageView(float Zoom, float CenterX, float CenterY);
