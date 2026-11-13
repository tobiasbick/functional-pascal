using System.Runtime.CompilerServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>One picture file or immutable tightly packed RGBA source.</summary>
/// <param name="Path">Picture filename, or null for RGBA pixels.</param>
/// <param name="Width">Pixel width for in-memory sources.</param>
/// <param name="Height">Pixel height for in-memory sources.</param>
/// <param name="Pixels">Four bytes per pixel in RGBA order.</param>
public readonly record struct CuiMediaImage(string? Path, int Width = 0, int Height = 0, byte[]? Pixels = null);

public sealed partial class CuiWindow
{
    private sealed record ImageRevision(long Value);

    private static long nextImageRevision;
    private static readonly ConditionalWeakTable<byte[], ImageRevision> imageRevisions = new();

    private static unsafe NativeMediaSource MediaSource(CuiMediaImage image, ChartMemory memory, byte* pixels)
    {
        if (image.Path is { } path)
        {
            Utf8.Validate(path, nameof(image));
            ArgumentException.ThrowIfNullOrEmpty(path);
            return new() { Path = memory.Text(path) };
        }

        if (image.Width <= 0 || image.Height <= 0 || image.Pixels is null ||
            (long)image.Width * image.Height != image.Pixels.LongLength / 4 || image.Pixels.LongLength % 4 != 0)
            throw new ArgumentException("An RGBA image needs positive dimensions and four bytes per pixel.", nameof(image));

        long version = imageRevisions.GetValue(image.Pixels,
            static _ => new(Interlocked.Increment(ref nextImageRevision))).Value;
        return new() { Pixels = pixels, Width = image.Width, Height = image.Height, Version = version };
    }
}
