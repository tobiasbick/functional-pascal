using System.Buffers.Binary;
using System.IO.Compression;

namespace Fpas.Ui.Gui.Cui.Tests;

internal static class PngAssertions
{
    internal static void HasRenderedContent(string path, int width, int height)
    {
        uint[] pixels = ReadPixels(path, width, height);
        Assert.True(Array.Exists(pixels, pixel => pixel != pixels[0]), "The captured native view contains only one color.");
    }

    internal static uint[] ReadPixels(string path, int width, int height)
    {
        byte[] png = File.ReadAllBytes(path);
        Assert.Equal(new byte[] { 137, 80, 78, 71, 13, 10, 26, 10 }, png[..8]);
        Assert.Equal(width, BinaryPrimitives.ReadInt32BigEndian(png.AsSpan(16, 4)));
        Assert.Equal(height, BinaryPrimitives.ReadInt32BigEndian(png.AsSpan(20, 4)));
        Assert.Equal(8, png[24]); // Eight-bit RGBA, noninterlaced, as written by CUI/stb.
        Assert.Equal(6, png[25]);
        Assert.Equal(0, png[28]);
        using var compressed = new MemoryStream();
        for (int offset = 8; offset < png.Length;)
        {
            int length = BinaryPrimitives.ReadInt32BigEndian(png.AsSpan(offset, 4));
            if (png.AsSpan(offset + 4, 4).SequenceEqual("IDAT"u8)) compressed.Write(png.AsSpan(offset + 8, length));
            offset += length + 12;
        }
        compressed.Position = 0;
        using var decoder = new ZLibStream(compressed, CompressionMode.Decompress);
        int stride = width * 4;
        var prior = new byte[stride];
        var row = new byte[stride];
        var pixels = new uint[width * height];
        for (int y = 0; y < height; y++)
        {
            int filter = decoder.ReadByte();
            Assert.InRange(filter, 0, 4);
            decoder.ReadExactly(row);
            for (int x = 0; x < stride; x++)
            {
                int left = x < 4 ? 0 : row[x - 4], above = prior[x], upperLeft = x < 4 ? 0 : prior[x - 4];
                int prediction = filter switch
                {
                    0 => 0, 1 => left, 2 => above, 3 => (left + above) / 2,
                    _ => Paeth(left, above, upperLeft)
                };
                row[x] = unchecked((byte)(row[x] + prediction));
            }
            for (int x = 0; x < width; x++)
                pixels[y * width + x] = BinaryPrimitives.ReadUInt32LittleEndian(row.AsSpan(x * 4, 4));
            (row, prior) = (prior, row);
        }
        Assert.Equal(-1, decoder.ReadByte());
        return pixels;
    }

    private static int Paeth(int left, int above, int upperLeft)
    {
        int prediction = left + above - upperLeft;
        int dl = Math.Abs(prediction - left), da = Math.Abs(prediction - above), dul = Math.Abs(prediction - upperLeft);
        return dl <= da && dl <= dul ? left : da <= dul ? above : upperLeft;
    }
}
