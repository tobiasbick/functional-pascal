// [Documentation](../../../docs/ui/gui/text-and-display/image.md)
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Media;

namespace Fpas.Ui.Gui.Cui.Tests.Media;

/// <summary>Compares a native subrectangle upload to a full upload of the same pixels.</summary>
public sealed class CuiImageUpdateTests
{
    /// <summary>A matched or skipped revision produces the same pixels as a fresh texture.</summary>
    /// <param name="renderer">Native texture and capture backend.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void RegionUpdatesMatchFullUploads(CuiRenderer renderer)
    {
        using var directory = new TemporaryDirectory();
        const int width = 400, height = 320;
        using var window = new CuiWindow("RGBA updates", width, height, renderer, continuous: true);
        Assert.Equal(renderer == CuiRenderer.Software, window.IsSoftware);
        window.SetReducedMotion(true);
        byte[] pixels = new byte[4 * 4 * 4];
        for (int alpha = 3; alpha < pixels.Length; alpha += 4) pixels[alpha] = 255;
        var original = Capture(new(null, 4, 4, pixels));
        var patch = (byte[])pixels.Clone();
        for (int y = 1; y <= 2; y++)
            for (int x = 1; x <= 2; x++)
                patch[(y * 4 + x) * 4] = 255;
        var partial = Capture(new(null, 4, 4, patch, pixels, 1, 1, 2, 2));
        Assert.False(original.SequenceEqual(partial));
        Assert.Equal(partial, Capture(new(null, 4, 4, (byte[])patch.Clone())));

        var skipped = (byte[])patch.Clone();
        skipped[1] = 255;
        var latest = (byte[])skipped.Clone();
        latest[^2] = 255;
        var caughtUp = Capture(new(null, 4, 4, latest, skipped, 3, 3, 1, 1));
        Assert.False(partial.SequenceEqual(caughtUp));
        Assert.Equal(caughtUp, Capture(new(null, 4, 4, (byte[])latest.Clone())));

        uint[] Capture(CuiMediaImage source)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(8, 0);
            window.DisplayImage("picture", source, 2, 80, 2, 80, 0, default);
            window.EndColumn();
            window.Present();
            string path = directory.File("image.png");
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, width, height);
        }
    }
}
