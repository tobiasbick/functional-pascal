using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>A native-compatible gradient stop.</summary>
/// <param name="Position">Ramp position.</param>
/// <param name="Rgba">Packed straight RGBA.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiGradientStop(float Position, uint Rgba);

/// <summary>Result of a native ramp edit.</summary>
/// <param name="Stops">Proposed stops.</param>
/// <param name="Selected">Selected index.</param>
/// <param name="Added">Inserted index.</param>
/// <param name="Removed">Removed index.</param>
public sealed record CuiGradientEdit(CuiGradientStop[] Stops, int Selected, int Added, int Removed);
