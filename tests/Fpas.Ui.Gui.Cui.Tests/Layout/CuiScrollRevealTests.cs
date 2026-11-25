// [Documentation](../../../docs/ui/gui/layout/README.md)
namespace Fpas.Ui.Gui.Cui.Tests.Layout;

/// <summary>Checks that a scroll container reveals a tracked control with the least movement.</summary>
public sealed class CuiScrollRevealTests
{
    /// <summary>Scrolls down to the clipped bottom edge, up to the clipped top edge, and not at all while visible.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void RevealMovesTheLeastThatShowsTheControl()
    {
        using var window = new CuiWindow("Scroll reveal", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        const string id = "reveal-list";

        Draw();
        var view = window.GetControlRectangle("view");
        Assert.True(window.GetControlRectangle("item/6").Y > view.Y + view.Height);

        var below = Settle("item/6");
        Assert.Equal(view.Y + view.Height, below.Y + below.Height, 0.5F);

        Assert.Equal(below, Settle("item/6"));

        Settle("item/1");
        Assert.Equal(view.Y, window.GetControlRectangle("item/1").Y, 0.5F);

        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentException>(() => window.ScrollReveal(id, "missing"));
        window.DiscardFrame();

        CuiRectangle Settle(string target)
        {
            Draw(target);
            Draw();
            return window.GetControlRectangle(target);
        }

        void Draw(string? reveal = null)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(12, 8);
            if (reveal is not null) window.ScrollReveal(id, reveal);
            window.BeginScroll(id, 112);
            window.TrackLastControl("view");
            for (int index = 0; index < 8; index++)
            {
                window.Button($"Item {index}");
                window.TrackLastControl($"item/{index}");
            }
            window.EndScroll();
            window.EndColumn();
            window.Present();
        }
    }
}
