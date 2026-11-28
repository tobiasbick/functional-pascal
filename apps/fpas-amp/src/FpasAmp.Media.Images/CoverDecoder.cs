// [Documentation](../../doc/media/images.md)
using System.Drawing;
using PhotoSauce.MagicScaler;
using PhotoSauce.MagicScaler.Transforms;
using PhotoSauce.NativeCodecs.Libjpeg;
using PhotoSauce.NativeCodecs.Libpng;

namespace FpasAmp.Media.Images;

/// <summary>Decodes JPEG and PNG images using MagicScaler.</summary>
public static class CoverDecoder
{
    static CoverDecoder()
    {
        CodecManager.Configure(codecs =>
        {
            codecs.UseLibjpeg();
            codecs.UseLibpng();
        });
    }

    /// <summary>Orients and fits an encoded picture into a transparent square.</summary>
    /// <param name="encoded">JPEG or PNG bytes, up to 16 MiB and 64 megapixels.</param>
    /// <param name="edge">Output width and height in pixels, from 1 to 8000.</param>
    /// <param name="cancellation">Checked before decoding and after copying pixels.</param>
    /// <returns>Detached straight-alpha RGBA pixels in row order.</returns>
    public static Bitmap Decode(byte[] encoded, int edge, CancellationToken cancellation)
    {
        ArgumentNullException.ThrowIfNull(encoded);
        ArgumentOutOfRangeException.ThrowIfLessThan(edge, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(edge, 8000);
        cancellation.ThrowIfCancellationRequested();
        if (encoded.Length > 16 * 1024 * 1024) throw new InvalidDataException("Image exceeds sixteen MiB.");
        try
        {
            var info = ImageFileInfo.Load(encoded);
            if (info.Frames.Count == 0 || info.Frames[0].Width < 1 || info.Frames[0].Height < 1 ||
                (long)info.Frames[0].Width * info.Frames[0].Height > 64_000_000)
                throw new InvalidDataException("Artwork exceeds supported image dimensions.");
            if (info.MimeType is not (ImageMimeTypes.Jpeg or ImageMimeTypes.Png))
                throw new InvalidDataException("Expected JPEG or PNG artwork.");
            cancellation.ThrowIfCancellationRequested();
            using var input = new MemoryStream(encoded, writable: false);
            var settings = new ProcessImageSettings
            {
                Width = edge, Height = edge, ResizeMode = CropScaleMode.Pad,
                MatteColor = Color.Transparent
            };
            settings.TrySetEncoderFormat(ImageMimeTypes.Png);
            using var pipeline = MagicImageProcessor.BuildPipeline(input, settings);
            pipeline.AddTransform(new FormatConversionTransform(PixelFormats.Bgra32bpp));
            var source = pipeline.PixelSource;
            var pixels = new byte[checked(source.Width * source.Height * 4)];
            source.CopyPixels(new Rectangle(0, 0, source.Width, source.Height), source.Width * 4, pixels);
            cancellation.ThrowIfCancellationRequested();
            for (int index = 0; index < pixels.Length; index += 4)
                (pixels[index], pixels[index + 2]) = (pixels[index + 2], pixels[index]);
            return new Bitmap(source.Width, source.Height, pixels);
        }
        catch (NotSupportedException error)
        {
            throw new InvalidDataException("Unsupported cover image.", error);
        }
    }
}
