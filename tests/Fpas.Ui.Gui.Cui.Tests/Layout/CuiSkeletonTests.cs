// [Documentation](../../../docs/ui/gui/feedback/skeleton.md)
namespace Fpas.Ui.Gui.Cui.Tests.Layout;

/// <summary>Checks native placeholder sizes, percentage allocation and ABI validation.</summary>
public sealed class CuiSkeletonTests
{
    /// <summary>Native width modes resize without converting percentages in managed code.</summary>
    /// <param name="renderer">Native renderer to exercise.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void WidthModesAndDefaultHeightFollowNativeLayout(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Skeleton sizing", 500, 260, renderer, continuous: true);
        foreach (int width in new[] { 500, 280, 600, 280 })
        {
            window.Resize(width, 260);
            Draw(); Draw();
            float available = window.Size.Width - 24;
            Check("fill", available, 14);
            Check("fixed", 40, 40);
            Check("percent", available * .45f, 12);
            Check("zero", 0, 14);
        }
        void Check(string id, float width, float height)
        {
            var box = window.GetControlRectangle(id);
            Assert.Equal(width, box.Width, precision: 1);
            Assert.Equal(height, box.Height);
            Assert.False(window.ControlFocused(id));
        }
        void Draw()
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginGroup(false, 12, 8, 4, false);
            Add("fill", 1, 0, 0);
            Add("fixed", 2, 40, 40);
            Add("percent", 3, 45, 12);
            Add("zero", 3, 0, 0);
            window.EndGroup();
            window.Present();
        }
        void Add(string id, int kind, float width, float height)
        {
            window.Skeleton(id, kind, width, height);
            window.SizeLast(false, 0, 0, 0, -1, 0, 4);
            window.TrackLastControl(id);
        }
    }

    /// <summary>Groups allocate native widths and gaps with either animation style.</summary>
    /// <param name="dither">Whether the groups draw ordered dots.</param>
    [GuiTheory]
    [InlineData(false)]
    [InlineData(true)]
    public void GroupsUseNativeWidthsAndGaps(bool dither)
    {
        using var window = new CuiWindow("Skeleton groups", 480, 240, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.True(window.NextFrame());
        window.BeginGroup(false, 12, 8, 4, false);
        Draw("fill", 1, 0, 0);
        Draw("fixed", 2, 140, 13);
        Draw("percent", 3, 60, 5);
        window.EndGroup();
        window.Present();
        float available = window.Size.Width - 24;
        Check("fill", available, 8);
        Check("fixed", 140, 13);
        Check("percent", available * .6f, 5);

        void Draw(string id, int widthKind, float width, float gap)
        {
            window.BeginSkeletonGroup(id, widthKind, width, gap, dither);
            window.Skeleton(id + "/first", height: 10);
            window.TrackLastControl(id + "/first");
            window.Skeleton(id + "/second", height: 8);
            window.TrackLastControl(id + "/second");
            window.EndSkeletonGroup();
        }

        void Check(string id, float width, float gap)
        {
            var first = window.GetControlRectangle(id + "/first");
            var second = window.GetControlRectangle(id + "/second");
            Assert.Equal(width, first.Width, precision: 1);
            Assert.Equal(gap, second.Y - first.Y - first.Height);
        }
    }

    /// <summary>General percentage sizes use the parent's inner extent and native bounds.</summary>
    /// <param name="renderer">Native renderer to exercise.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void SharedPercentagesRespectParentPaddingAndBounds(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Percentage sizing", 400, 240, renderer, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginGroup(true, 20, 0, 0, false);
        window.Text("A"); window.SizeLast(true, 3, 25, 0, -1, 0, 0); window.TrackLastControl("width");
        window.Text("B"); window.SizeLast(true, 3, 50, 0, 80, 0, 0); window.TrackLastControl("bounded");
        window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 0);
        window.Present();
        Assert.Equal(90, window.GetControlRectangle("width").Width, precision: 1);
        Assert.Equal(80, window.GetControlRectangle("bounded").Width);
        Assert.True(window.NextFrame());
        window.BeginGroup(false, 20, 0, 4, false);
        window.Text("A"); window.SizeLast(false, 3, 25, 0, -1, 0, 4); window.TrackLastControl("height");
        window.Text("B"); window.SizeLast(false, 1, 0, 0, -1, 1, 4);
        window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 0);
        window.Present();
        Assert.Equal(50, window.GetControlRectangle("height").Height, precision: 1);
    }

    /// <summary>Invalid modes, percentages, bounds, heights and growth combinations leave the frame usable.</summary>
    [GuiFact]
    public void InvalidSizesDoNotCorruptTheFrame()
    {
        using var window = new CuiWindow("Skeleton validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        foreach (int kind in new[] { 0, -1, 4 })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Skeleton("bad", kind));
        foreach (float value in new[] { -1, float.NaN, float.PositiveInfinity, 101 })
        {
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Skeleton("bad", 3, value));
            Assert.Throws<ArgumentOutOfRangeException>(() => window.SizeLast(true, 3, value, 0, -1, 0, 0));
        }
        Assert.Throws<ArgumentOutOfRangeException>(() => window.SizeLast(true, 3, 50, 0, -1, 1, 0));
        Assert.Throws<ArgumentOutOfRangeException>("maximum", () => window.SizeLast(true, 0, 0, 0, -.5f, 0, 0));
        Assert.Throws<ArgumentOutOfRangeException>("maximum", () => window.SizeLast(true, 0, 0, 10, 5, 0, 0));
        foreach (float height in new[] { -1, float.NaN, float.PositiveInfinity })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Skeleton("bad", height: height));
        window.Skeleton("valid"); window.Present();
    }
}
