// [Documentation](../../../../docs/ui/gui/text-and-display/image.md)
using System.Runtime.CompilerServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Media;

/// <summary>One picture file or immutable tightly packed RGBA source.</summary>
/// <param name="Path">Picture filename, or null for RGBA pixels.</param>
/// <param name="Width">Pixel width for in-memory sources.</param>
/// <param name="Height">Pixel height for in-memory sources.</param>
/// <param name="Pixels">Four bytes per pixel in RGBA order.</param>
/// <param name="Previous">Snapshot on which the optional update rectangle is based.</param>
/// <param name="X">Updated rectangle's left pixel.</param>
/// <param name="Y">Updated rectangle's top pixel.</param>
/// <param name="RegionWidth">Updated pixel columns.</param>
/// <param name="RegionHeight">Updated pixel rows.</param>
public readonly record struct CuiMediaImage(string? Path, int Width = 0, int Height = 0, byte[]? Pixels = null,
    byte[]? Previous = null, int X = 0, int Y = 0, int RegionWidth = 0, int RegionHeight = 0);
