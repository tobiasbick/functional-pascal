using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Resources;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui.Tests.Resources;

/// <summary>Verifies window-owned resources and theme customization without deployment.</summary>
public sealed class CuiFontThemeTests
{
    /// <summary>Copies font bytes, switches/restores roles and enforces context/thread/frame lifetimes.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void FontFacesAreCopiedAndOwnedByTheirWindow(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Owned fonts", 400, 200, renderer, continuous: true);
        window.SetReducedMotion(true);
        float original = Width();
        byte[] bytes = FontFixture.Read();
        var face = Assert.IsType<CuiFontFace>(window.LoadFont(bytes));
        Array.Clear(bytes); GC.Collect();
        window.SetFontRole(3, face);
        Assert.Throws<ArgumentException>(() => window.SetFontRole(4, face));
        float replaced = Width();
        Assert.NotEqual(original, replaced);
        window.SetFontRole(3, null);
        Assert.Equal(original, Width());
        Assert.Null(window.LoadFont([]));
        window.SetFontRole(3, face);
        float automaticFallback = Width("\uE13D");
        var icons = Assert.IsType<CuiFontFace>(window.LoadFont(FontFixture.Read("lucide")));
        window.AddFontFallback(face, icons);
        // A direct face takes priority over CUI's automatically scaled icon-role fallback.
        Assert.NotEqual(automaticFallback, Width("\uE13D"));
        Assert.Throws<ArgumentException>(() => window.AddFontFallback(face, icons));
        for (int i = 0; i < 3; i++) window.AddFontFallback(face, Assert.IsType<CuiFontFace>(window.LoadFont(FontFixture.Read())));
        window.SetFontRole(3, null); window.SetFontRole(4, null);
        Assert.Throws<InvalidOperationException>(() => window.AddFontFallback(face, Assert.IsType<CuiFontFace>(window.LoadFont(FontFixture.Read()))));
        Assert.Throws<ArgumentException>(() => window.AddFontFallback(face, face));
        using (var other = new CuiWindow("Another font context", renderer: CuiRenderer.Software))
            Assert.Throws<ArgumentException>(() => other.SetFontRole(0, face));
        Assert.IsType<InvalidOperationException>(Task.Factory.StartNew(
            () => Record.Exception(() => window.SetFontRole(0, face)), CancellationToken.None,
            TaskCreationOptions.LongRunning, TaskScheduler.Default).GetAwaiter().GetResult());
        Assert.True(window.NextFrame());
        Assert.Throws<InvalidOperationException>(() => window.LoadFont(FontFixture.Read()));
        window.DiscardFrame();
        Assert.False(window.LoadEmojiFont([]));
        Assert.True(window.LoadEmojiFont(FontFixture.Read()));
        _ = window.LoadSystemEmojiFont();
        _ = window.SupportsEmoji("👩‍💻");
        Assert.Throws<ArgumentException>(() => window.SupportsEmoji("bad\0text"));
        window.Dispose();
        Assert.Throws<ObjectDisposedException>(() => window.SetFontRole(0, face));

        float Width(string text = "iiiiWWWW")
        {
            float width = 0;
            Assert.True(window.NextFrame());
            window.DrawingCanvas("font", 1, 0, 2, 80, [], true, out _, out _, out _, out _, false, out _, out _, p =>
            { width = p.Measure(text, 3, 20).Width; p.Text(0, 0, "Grüße 🎵", 3, 20, uint.MaxValue); });
            window.Present(); return width;
        }
    }

    /// <summary>Preserves custom palettes, round-trips all metrics and restores native appearance defaults.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ThemeMetricsAndAppearanceKeepApplicationColors(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Native theme metrics", 440, 250, renderer, continuous: true);
        window.SetReducedMotion(true);
        uint[] colors = window.ReadTheme();
        colors[0] = 0xff352719;
        var metrics = window.ReadThemeMetrics();
        metrics[0] = 3; metrics[3] = 27; metrics[4] = 60; metrics[6] = 10; metrics[10] = 40;
        metrics[15] = 1.5f; metrics[16] = 2; metrics[17] = 4; metrics[18] = -2; metrics[19] = .2f;
        window.FollowSystemTheme(true);
        window.SetTheme(true, 2, colors, metrics);
        Assert.Equal(colors, window.ReadTheme());
        Assert.Equal(metrics, window.ReadThemeMetrics());
        Assert.True(window.NextFrame());
        window.BeginColumn(8, 0);
        window.DrawingCanvas("scale", 1, 0, 2, 100, [], true, out _, out _, out _, out _, false, out _, out _, p =>
        {
            Assert.Equal(1.5f, p.FontScale);
            Assert.True(p.Measure("Scaled", 0, 20 * p.FontScale).Width > p.Measure("Scaled", 0, 20).Width);
            p.Text(0, 0, "Scaled", 0, 20 * p.FontScale, uint.MaxValue);
        });
        window.Button("Theme height"); window.TrackLastControl("height");
        window.EndColumn(); window.Present();
        Assert.Equal(60, window.GetControlRectangle("height").Height);
        window.FollowSystemTheme(false);
        window.SetTheme(false, 0, colors);
        float[] defaults = window.ReadThemeMetrics();
        Assert.Equal(10, defaults[0]);
        Assert.Equal(1.5f, defaults[15]);
        Assert.Equal(0, defaults[14]);
        float[] bad = (float[])metrics.Clone(); bad[15] = 0;
        Assert.Throws<ArgumentException>(() => window.SetTheme(false, 0, colors, bad));
        Assert.Equal(defaults, window.ReadThemeMetrics());
        bad[15] = 1; bad[19] = float.NaN;
        Assert.Throws<ArgumentException>(() => window.SetTheme(false, 0, colors, bad));
        metrics[14] = 1.5f;
        window.SetTheme(true, 2, colors, metrics);
        Assert.Equal(1.5f, window.ReadThemeMetrics()[14]);
    }

    /// <summary>Copies partial Unicode captions, permits replacement/reset and validates before mutation.</summary>
    [GuiFact]
    public void NativeCaptionTablesOwnTheirStrings()
    {
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Owned captions", 500, 440, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        byte[] english = Capture("english");
        var names = new string?[12]; names[0] = "Januar 日本";
        window.SetStrings(new(Months: names, Today: "Heute 🎵"));
        byte[] translated = Capture("translated");
        Assert.NotEqual(english, translated);
        names[0] = "Changed"; GC.Collect();
        Assert.Equal(translated, Capture("copied"));
        Assert.Throws<ArgumentException>(() => window.SetStrings(new(Weekdays: ["Monday"])));
        Assert.Throws<ArgumentException>(() => window.SetStrings(new(Today: "invalid\0text")));
        Assert.Equal(translated, Capture("rejected"));
        Assert.True(window.NextFrame());
        Assert.Throws<InvalidOperationException>(() => window.SetStrings(null));
        window.DiscardFrame();
        window.SetStrings(new(Months: names));
        Assert.NotEqual(translated, Capture("replacement"));
        window.SetStrings(null);
        Assert.Equal(english, Capture("reset"));

        byte[] Capture(string name)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.Calendar("calendar", new DateOnly(2026, 1, 14), true, 0, null);
            window.Present();
            string path = directory.File(name + ".png");
            window.CapturePng(path);
            return File.ReadAllBytes(path);
        }
    }

    /// <summary>Uses CUI's color conversion, interpolation and alpha helpers.</summary>
    [Fact]
    public void ColorHelpersUseNativeColorSpaces()
    {
        Assert.Equal(0xff000000u, CuiColors.Oklch(0, 0, 0));
        Assert.Equal(0xffffffffu, CuiColors.Oklch(1, 0, 0));
        Assert.Equal(0x80123456u, CuiColors.Alpha(0xff123456, .5f));
        Assert.Equal(0xff123456u, CuiColors.Mix(0xff123456, 0xff123456, .5f));
        Assert.Throws<ArgumentOutOfRangeException>(() => CuiColors.Oklch(1, float.NaN, 0));
        Assert.Throws<ArgumentOutOfRangeException>(() => CuiColors.Mix(0, 0, 2));
    }
}
