// [Documentation](../../Fixtures/gui-gallery-reference/README.md)
namespace Fpas.Ui.Gui.Cui.Tests.Packaging;

internal static class GalleryReferenceTests
{
    private sealed record Crop(string Page, int PageWidth, int PageHeight, string Name,
        int X, int Y, int Width, int Height, int MaxMeanError);

    private static readonly Crop[] Crops =
    [
        new(".actions-complete", 1000, 1440, "actions-bold", 281, 317, 35, 38, 4),
        new(".selection", 1100, 1360, "selection-switch", 699, 238, 42, 26, 5),
        new(".ruler-histogram", 1000, 1600, "picker-ruler", 422, 1367, 50, 32, 7),
        new(".disclosure-feedback", 1100, 1500, "disclosure-check", 297, 1319, 31, 29, 9),
        new(".audio", 1201, 800, "audio-cutoff", 327, 211, 57, 48, 11),
        new(".text-fields", 1000, 760, "input-search", 626, 322, 35, 42, 7),
        new("-canvas-1100", 1100, 1700, "format-color", 280, 354, 65, 49, 3),
        new("-canvas-1100", 1100, 1700, "canvas-tools", 998, 736, 54, 119, 3),
        new("-creative-1100", 1100, 1700, "ratio-label", 280, 355, 73, 55, 6),
        new("-creative-1100", 1100, 1700, "palette-boundary", 421, 889, 30, 43, 5),
    ];

    internal static void Match(string capture)
    {
        foreach (var page in Crops.GroupBy(crop => crop.Page))
        {
            var first = page.First();
            var actual = PngAssertions.ReadPixels(capture + page.Key + ".png", first.PageWidth, first.PageHeight);
            foreach (var crop in page)
            {
                string path = Path.Combine(AppContext.BaseDirectory, "Fixtures", "gallery-reference", crop.Name + ".png");
                var reference = PngAssertions.ReadPixels(path, crop.Width, crop.Height);
                long totalError = 0;
                for (int y = 0; y < crop.Height; y++)
                    for (int x = 0; x < crop.Width; x++)
                    {
                        uint pixel = actual[(crop.Y + y) * crop.PageWidth + crop.X + x];
                        uint expected = reference[y * crop.Width + x];
                        totalError += Math.Max(Difference(0), Math.Max(Difference(8), Difference(16)));

                        int Difference(int shift) =>
                            Math.Abs((int)((pixel >> shift) & 255) - (int)((expected >> shift) & 255));
                    }
                double meanError = (double)totalError / reference.Length;
                Assert.True(meanError <= crop.MaxMeanError,
                    $"CUI reference {crop.Name} differs at ({crop.X}, {crop.Y}) on {page.Key}: " +
                    $"mean error {meanError:F2}, allowed {crop.MaxMeanError}.");
            }
        }
    }
}
