// [Documentation](../../../../docs/ui/gui/text-and-display/image.md)
using Fpas.Ui.Gui.Cui.Interop.Media;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Media;
using System.Runtime.CompilerServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

        var source = new NativeMediaSource { Pixels = pixels, Width = image.Width, Height = image.Height, Version = Revision(image.Pixels) };
        if (image.Previous is { } previous)
        {
            if (previous.Length != image.Pixels.Length || image.X < 0 || image.Y < 0 ||
                image.RegionWidth <= 0 || image.RegionHeight <= 0 ||
                image.X > image.Width - image.RegionWidth || image.Y > image.Height - image.RegionHeight)
                throw new ArgumentException("Invalid RGBA update rectangle or previous snapshot.", nameof(image));
            source.Previous = Revision(previous);
            (source.X, source.Y, source.RegionWidth, source.RegionHeight) = (image.X, image.Y, image.RegionWidth, image.RegionHeight);
        }
        return source;
    }

    private static long Revision(byte[] pixels) => imageRevisions.GetValue(pixels,
            static _ => new(Interlocked.Increment(ref nextImageRevision))).Value;

}
