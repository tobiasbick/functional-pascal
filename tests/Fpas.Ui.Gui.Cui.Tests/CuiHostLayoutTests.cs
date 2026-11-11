using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks shared flex allocation against finalized native geometry.</summary>
public sealed class CuiHostLayoutTests
{
    /// <summary>Uses CUI weighted growth from zero, frozen bounds and nonshrinking fixed extents.</summary>
    /// <param name="renderer">Renderer whose native layout is exercised.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void HostSizingUsesCuiGrowthBoundsAndFixedExtents(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Shared flex", 400, 240, renderer, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginGroup(false, 0, 0, 0, false);
        Row("grow", 1, 0, 1, 0, 1, 3, -1);
        Row("cap", 1, 0, 1, 0, 1, 3, 80);
        Row("mixed", 2, 100, 1, 0, 0, 1, -1);
        Row("fixed", 2, 300, 2, 300, 0, 0, -1);
        window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 0);
        window.Present();
        Check("grow", 100, 300); Check("cap", 80, 320); Check("mixed", 100, 300); Check("fixed", 300, 300);

        void Row(string id, int kindA, float a, int kindB, float b, float growA, float growB, float cap)
        {
            window.BeginGroup(true, 0, 0, 0, false);
            window.Button(id + "a", "A", 0, 0, true, false, false, false, "", "", "", "", false, -1, 0);
            window.SizeLast(true, kindA, a, 0, cap, growA, 0); window.TrackLastControl(id + "a");
            window.Button(id + "b", "B", 0, 0, true, false, false, false, "", "", "", "", false, -1, 0);
            window.SizeLast(true, kindB, b, 0, -1, growB, 0); window.TrackLastControl(id + "b");
            window.EndGroup(); window.SizeLast(false, 2, 50, 0, -1, 0, 0);
        }
        void Check(string id, float a, float b)
        {
            Assert.Equal(a, window.GetControlRectangle(id + "a").Width, precision: 1);
            Assert.Equal(b, window.GetControlRectangle(id + "b").Width, precision: 1);
        }
    }

    /// <summary>Native text measurements share shrinkage while a fixed sibling retains its width.</summary>
    [GuiFact]
    public void NativeContentShrinksBesideFixedSibling()
    {
        using var window = new CuiWindow("Content sizing", 400, 240, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginGroup(true, 0, 0, 0, false);
        window.Text(string.Join(" ", Enumerable.Repeat("Wide", 12))); window.TrackLastControl("a");
        window.SizeLast(true, 0, 0, 0, -1, 0, 0);
        window.Text(string.Join(" ", Enumerable.Repeat("Wide", 4))); window.TrackLastControl("b");
        window.SizeLast(true, 0, 0, 0, -1, 0, 0);
        window.Text("Fixed"); window.TrackLastControl("fixed"); window.SizeLast(true, 2, 200, 0, -1, 0, 0);
        window.EndGroup(); window.SizeLast(false, 1, 0, 0, -1, 1, 4);
        window.Present();
        float a = window.GetControlRectangle("a").Width, b = window.GetControlRectangle("b").Width;
        Assert.Equal(200, a + b, precision: 1);
        Assert.True(a > b && b > 0);
        Assert.Equal(200, window.GetControlRectangle("fixed").Width);
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
            { window.Text("Scrollable line"); window.SizeLast(false, 0, 0, 0, -1, 0, 0); }
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
