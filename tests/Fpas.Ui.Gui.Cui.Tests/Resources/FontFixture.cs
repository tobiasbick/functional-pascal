using System.Globalization;
using System.IO.Compression;

namespace Fpas.Ui.Gui.Cui.Tests.Resources;

// Reuse CUI's licensed embedded face instead of adding another binary dependency.
internal static class FontFixture
{
    internal static byte[] Read(string name = "inter_regular")
    {
        string source = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Fixtures", "CuiFonts.c"));
        int start = source.IndexOf("const unsigned char cui_font_" + name + "[", StringComparison.Ordinal);
        start = source.IndexOf('{', start) + 1;
        int end = source.IndexOf('}', start);
        byte[] packed = source[start..end].Split([',', ' ', '\r', '\n'], StringSplitOptions.RemoveEmptyEntries)
            .Select(x => byte.Parse(x, CultureInfo.InvariantCulture)).ToArray();
        using var input = new MemoryStream(packed);
        using var inflate = new DeflateStream(input, CompressionMode.Decompress);
        using var output = new MemoryStream();
        inflate.CopyTo(output);
        return output.ToArray();
    }
}
