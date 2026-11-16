using System.Drawing;
using PhotoSauce.MagicScaler;
using PhotoSauce.MagicScaler.Transforms;
using PhotoSauce.NativeCodecs.Libjpeg;
using PhotoSauce.NativeCodecs.Libpng;

namespace FpasAmp.Imaging;

// The only image-processing boundary: MagicScaler owns decoding and resampling.
internal static class CoverDecoder
{
    static CoverDecoder()
    {
        CodecManager.Configure(codecs =>
        {
            codecs.UseLibjpeg();
            codecs.UseLibpng();
        });
    }

    internal static Bitmap Decode(byte[] encoded, int edge, CancellationToken cancellation)
    {
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
