// [Documentation](../../../docs/ui/gui/text-and-display/icon.md)
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Verifies native icon size and colors in rendered frames.</summary>
public sealed class CuiIconTests
{
    /// <summary>Renders foreground, live theme colors, and explicit RGBA while preserving layout at zero alpha.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void IconUsesSizeAndActiveColors(CuiRenderer renderer)
    {
        const int width = 240, height = 120, accentToken = 11;
        const string glyph = "\uE0CC";
        using var window = new CuiWindow("Native icons", width, height, renderer, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);

        var defaultIcon = Capture("default", 0);
        Assert.Equal(defaultIcon.Pixels, Capture("default-size", 16).Pixels);
        var largeIcon = Capture("large", 36);
        Assert.True(largeIcon.CaptionY > defaultIcon.CaptionY);
        Assert.NotEqual(defaultIcon.Pixels, largeIcon.Pixels);

        var palette = window.ReadTheme();
        palette[accentToken - 1] = 0xff0000e0;
        window.SetTheme(false, 2, palette);
        var themed = Capture("theme", 36, themeToken: accentToken);
        Assert.Equal(Capture("explicit", 36, palette[accentToken - 1]).Pixels, themed.Pixels);
        Assert.NotEqual(themed.Pixels, Capture("half-opacity", 36, 0x800000e0).Pixels);
        palette[accentToken - 1] = 0xffe00000;
        window.SetTheme(true, 2, palette);
        var updated = Capture("updated-theme", 36, themeToken: accentToken);
        Assert.Equal(Capture("updated-explicit", 36, palette[accentToken - 1]).Pixels, updated.Pixels);
        Assert.NotEqual(themed.Pixels, updated.Pixels);

        var transparent = Capture("transparent", 36, 0u);
        Assert.Equal(updated.CaptionY, transparent.CaptionY);
        Assert.NotEqual(Capture("foreground", 36).Pixels, transparent.Pixels);
        Assert.DoesNotContain(transparent.Pixels.Take((int)transparent.CaptionY * width),
            pixel => pixel != transparent.Pixels[0]);

        (uint[] Pixels, float CaptionY) Capture(string name, float size, uint? color = null, int themeToken = 0)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(8, 0);
            window.DisplayIcon(glyph, size, color, themeToken);
            window.Button("Below the icon");
            window.TrackLastControl("caption");
            window.EndColumn();
            window.Present();
            string path = directory.File(name + ".png");
            window.CapturePng(path);
            return (PngAssertions.ReadPixels(path, width, height), window.GetControlRectangle("caption").Y);
        }
    }
}
