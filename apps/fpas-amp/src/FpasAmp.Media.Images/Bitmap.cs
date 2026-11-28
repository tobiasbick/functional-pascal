// [Documentation](../../doc/media/images.md)
namespace FpasAmp.Media.Images;

/// <summary>A detached square thumbnail. Consumers must retain pixels unchanged.</summary>
/// <param name="Width">Pixel width.</param>
/// <param name="Height">Pixel height.</param>
/// <param name="Pixels">Straight-alpha RGBA pixels in row order.</param>
public sealed record Bitmap(int Width, int Height, byte[] Pixels);
