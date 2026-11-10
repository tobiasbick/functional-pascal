using Fpas.Tests.Support;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Checks shared flex allocation against finalized native geometry.</summary>
public sealed class CuiHostLayoutTests
{
    /// <summary>Preserves intrinsic canvas dimensions and paints the copied operations.</summary>
    [GuiFact]
    public void ContentSizingKeepsIntrinsicCanvasExtentAndPixels()
    {
        using var window = new CuiWindow("Intrinsic canvas", 400, 240, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginGroup(false, 0, 0, 0, false);
        window.Canvas("canvas", 80, 60, [new CuiPaint { C = 80, D = 60, Color = 0xff0000ff }]);
        window.TrackLastControl("canvas"); window.SizeLast(false, 0, 0, 0, -1, 0, 0);
        window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 0);
        window.Present();
        var bounds = window.GetControlRectangle("canvas");
        Assert.Equal(80, bounds.Width); Assert.Equal(60, bounds.Height);
        using var directory = new TemporaryDirectory();
        string capture = directory.File("canvas.png"); window.CapturePng(capture);
        uint[] pixels = PngAssertions.ReadPixels(capture, 400, 240);
        Assert.Equal(0xff0000ffu, pixels[((int)bounds.Y + 10) * 400 + (int)bounds.X + 10]);
    }
    /// <summary>Preserves basis, growth caps and explicit shrink weights.</summary>
    /// <param name="renderer">Renderer whose native layout is exercised.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void SharedFlexPreservesBasisBoundsAndShrinkWeights(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Shared flex", 400, 240, renderer, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginGroup(false, 0, 0, 0, false);
        Row("grow", 100, 100, 1, 3, -1, 1, 1);
        Row("cap", 100, 100, 1, 3, 130, 1, 1);
        Row("shrink", 300, 300, 0, 0, -1, 0, 2);
        window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 0);
        window.Present();
        Check("grow", 150, 250); Check("cap", 130, 270); Check("shrink", 300, 100);

        void Row(string id, float a, float b, float growA, float growB, float cap, float shrinkA, float shrinkB)
        {
            window.BeginGroup(true, 0, 0, 0, false);
            window.Control(id + "a", "A", 0, true, false, 0, false);
            window.SizeLast(true, 2, a, 0, cap, growA, 0, shrinkA); window.TrackLastControl(id + "a");
            window.Control(id + "b", "B", 0, true, false, 0, false);
            window.SizeLast(true, 2, b, 0, -1, growB, 0, shrinkB); window.TrackLastControl(id + "b");
            window.EndGroup(); window.SizeLast(false, 2, 50, 0, -1, 0, 0);
        }
        void Check(string id, float a, float b)
        {
            Assert.Equal(a, window.GetControlRectangle(id + "a").Width, precision: 1);
            Assert.Equal(b, window.GetControlRectangle(id + "b").Width, precision: 1);
        }
    }

    /// <summary>Reconciles End against current-frame content and routes wheel input without a scrollbar.</summary>
    /// <param name="scrollbar">Whether native scrollbar painting is requested.</param>
    [GuiTheory]
    [InlineData(true)]
    [InlineData(false)]
    public void ScrollEndFollowsContentAndOptionalScrollbarStillAcceptsWheel(bool scrollbar)
    {
        using var window = new CuiWindow("Controlled scrolling", 400, 240, CuiRenderer.Software, continuous: true);
        float previous = 0;
        for (int frame = 0; frame < 3; frame++)
        {
            Assert.True(window.NextFrame());
            if (frame == 2) { window.MovePointer(20, 20); window.Wheel(-1); }
            window.BeginGroup(false, 0, 0, 0, false);
            window.ScrollState("scroll", frame == 2 ? 0 : 1, 0, out _, out _);
            window.BeginViewport("scroll", scrollbar);
            window.BeginGroup(false, 0, 0, 0, false);
            for (int line = 0; line < (frame == 0 ? 30 : 40); line++)
            { window.Text("Scrollable line"); window.SizeLast(false, 0, 0, 0, -1, 0, 0, 0); }
            window.EndGroup(); window.SizeLast(false, 0, 0, 0, -1, 0, 0);
            window.EndScroll(); window.TrackLastControl("scroll"); window.SizeLast(false, 2, 100, 0, -1, 0, 0);
            window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 0);
            window.ScrollState("scroll", -1, 0, out float proposal, out _);
            if (frame == 2) Assert.True(proposal > 0, "Wheel input did not reach the viewport.");
            window.Present();
            window.ScrollMetrics("scroll", out float offset, out float maximum);
            Assert.True(maximum > 0);
            if (frame < 2) { Assert.Equal(maximum, offset, precision: 1); Assert.True(offset > previous); previous = offset; }
            else Assert.Equal(0, offset);
        }
    }
}
