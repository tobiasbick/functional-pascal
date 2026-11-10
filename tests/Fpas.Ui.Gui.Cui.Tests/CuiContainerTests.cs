namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises native scrolling through the managed window.</summary>
public sealed class CuiContainerTests
{
    /// <summary>Checks that scrolling moves the contents while the following widget stays in place.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void ScrollToMovesContentWithinTheFixedViewport()
    {
        using var window = new CuiWindow("Scroll container", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        const string id = "Aktivität-🧭";
        const float height = 112;
        const float padding = 12;
        const float gap = 8;
        const float offset = 80;

        Draw();
        var initial = Draw(offset);
        var scrolled = Draw();

        Assert.True(initial.Item.Width > 0);
        Assert.Equal(padding + height + gap, initial.Following.Y);
        Assert.Equal(initial.Item.Y - offset, scrolled.Item.Y);
        Assert.Equal(initial.Following.Y, scrolled.Following.Y);

        (CuiRectangle Item, CuiRectangle Following) Draw(float? scrollOffset = null)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(padding, gap);
            if (scrollOffset is float y) window.ScrollTo(id, y);
            window.BeginScroll(id, height);
            CuiRectangle item = default;
            for (int index = 0; index < 8; index++)
            {
                window.Button($"Item {index}");
                if (index == 2) item = window.LastRectangle;
            }
            window.EndScroll();
            window.Button("After scrolling");
            CuiRectangle following = window.LastRectangle;
            window.EndColumn();
            return (item, following);
        }
    }
}
