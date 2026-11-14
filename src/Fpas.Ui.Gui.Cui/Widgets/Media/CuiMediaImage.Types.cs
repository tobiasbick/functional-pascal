using System.Runtime.CompilerServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Media;

/// <summary>One picture file or immutable tightly packed RGBA source.</summary>
/// <param name="Path">Picture filename, or null for RGBA pixels.</param>
/// <param name="Width">Pixel width for in-memory sources.</param>
/// <param name="Height">Pixel height for in-memory sources.</param>
/// <param name="Pixels">Four bytes per pixel in RGBA order.</param>
public readonly record struct CuiMediaImage(string? Path, int Width = 0, int Height = 0, byte[]? Pixels = null);
