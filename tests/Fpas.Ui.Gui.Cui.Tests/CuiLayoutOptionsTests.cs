namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks native spacing, layered boxes, logical insets and two-axis scrolling.</summary>
public sealed class CuiLayoutOptionsTests
{
    /// <summary>Native RTL mirrors start/end padding and spacer placement after resizing.</summary>
    /// <param name="renderer">Native renderer.</param>
    /// <param name="rtl">Whether to mirror the row.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, false)]
    [InlineData(CuiRenderer.Software, true)]
    [InlineData(CuiRenderer.OpenGl, false)]
    [InlineData(CuiRenderer.OpenGl, true)]
    public void InsetsAndNativeSpacingFollowReadingDirection(CuiRenderer renderer, bool rtl)
    {
        const int height = 300;
        using var window = new CuiWindow("Native spacing", 520, height, renderer, continuous: true);
        // CUI's default minimum window size is 360 × 280 logical units.
        foreach (int width in new[] { 520, 380, 520 })
        {
            window.Resize(width, height);
            Draw(); Draw();
            Assert.Equal(width, window.Size.Width);
            var first = window.GetControlRectangle("first");
            var last = window.GetControlRectangle("last");
            Assert.Equal(rtl ? width - 100 : 40, first.X);
            Assert.Equal(rtl ? 20 : width - 100, last.X);
            Assert.Equal(10, first.Y);
            Assert.Equal(10, last.Y);
        }
        void Draw()
        {
            Assert.True(window.NextFrame());
            window.BeginGroup(true, 0, 0, 1, false);
            window.LayoutLast(10, 20, 30, 40, rtl ? 2 : 1);
            window.Skeleton("first", 2, 60, 20); window.TrackLastControl("first");
            window.Gap(10); window.Spacer();
            window.Skeleton("last", 2, 80, 20); window.TrackLastControl("last");
            window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 0);
            window.Present();
        }
    }

    /// <summary>Native stack alignment puts independently sized children on top of each other.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void LayersRespectIndependentAxesAndInsets(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Layered boxes", 400, 200, renderer, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginLayers(3, 2);
        window.LayoutLast(5, 7, 11, 13);
        window.Skeleton("back", 2, 160, 60); window.TrackLastControl("back");
        window.Skeleton("front", 2, 40, 20); window.TrackLastControl("front");
        window.EndGroup();
        window.SizeLast(true, 2, 200, 0, -1, 0, 0);
        window.SizeLast(false, 2, 100, 0, -1, 0, 0);
        window.Present();
        var back = window.GetControlRectangle("back");
        var front = window.GetControlRectangle("front");
        Assert.Equal(153, front.X); Assert.Equal(69, front.Y);
        Assert.Equal(back.X + back.Width, front.X + front.Width);
        Assert.Equal(back.Y + back.Height, front.Y + front.Height);
    }

    /// <summary>Horizontal wheel input reaches visible and hidden native scrollbars without moving a disabled view.</summary>
    /// <param name="bar">Whether scrollbars are visible.</param>
    [GuiTheory]
    [InlineData(true)]
    [InlineData(false)]
    public void HorizontalScrollingUsesNativeInputAndClipsContent(bool bar)
    {
        using var window = new CuiWindow("Horizontal overflow", 400, 200, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        for (int frame = 0; frame < 3; frame++) Draw(false, true);
        float before = window.GetControlRectangle("item").X;
        for (int frame = 0; frame < 3; frame++) Draw(true, true);
        float after = window.GetControlRectangle("item").X;
        Assert.True(after < before, $"The native horizontal viewport did not scroll: {before}, {after}.");
        for (int frame = 0; frame < 3; frame++) Draw(true, false);
        Assert.Equal(after, window.GetControlRectangle("item").X);
        void Draw(bool wheel, bool enabled)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(100, 80);
            if (wheel) window.Wheel(-1, 0);
            window.BeginViewport("scroll", bar, enabled, true, true);
            window.BeginGroup(true, 0, 0, 0, false);
            window.Skeleton("item", 2, 900, 80); window.TrackLastControl("item");
            window.EndGroup();
            window.EndScroll();
            window.Present();
        }
    }

    /// <summary>Invalid native layout options fail before mutating the frame.</summary>
    [GuiFact]
    public void InvalidLayoutOptionsLeaveTheFrameUsable()
    {
        using var window = new CuiWindow("Layout validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        foreach (float value in new[] { -1, float.NaN, float.PositiveInfinity })
        {
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Gap(value));
            Assert.Throws<ArgumentOutOfRangeException>(() => window.LayoutLast(value, 0, 0, 0));
        }
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BeginLayers(5));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.BeginLayers(justification: 5));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.LayoutLast(0, 0, 0, 0, 3));
        window.BeginLayers(); window.EndGroup(); window.Present();
    }
}
