// [Documentation](../../../docs/ui/gui/selection/chip.md)
namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks native chip activation and rating limits.</summary>
public sealed class CuiChipRatingTests
{
    /// <summary>Activates a stateless chip without drawing a selected check mark.</summary>
    [GuiFact]
    public void ChipKeepsItsAppearanceDuringStatelessActivation()
    {
        using var window = new CuiWindow("Chip", 480, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);

        Assert.False(Draw(null, focus: true));
        float width = window.GetControlRectangle("chip").Width;
        Assert.True(Draw(null, key: 2));
        Assert.Equal(width, window.GetControlRectangle("chip").Width);
        Assert.True(window.ControlFocused("chip"));
        Assert.False(Draw(null, icon: ""));
        Assert.True(window.GetControlRectangle("chip").Width > width);

        Assert.True(Draw(false, key: 6));
        Assert.True(window.GetControlRectangle("chip").Width > width);
        Assert.False(Draw(null, key: 2, enabled: false));
        Assert.False(window.ControlFocused("chip"));

        bool Draw(bool? selected, int key = 0, bool enabled = true, bool focus = false, string? icon = null)
        {
            Assert.True(window.NextFrame());
            if (key != 0)
            {
                window.SetKey(key, true);
                window.SetKey(key, false);
            }
            window.BeginColumn();
            bool activated = window.Chip("chip", "Design ## literal", icon, selected, enabled, focus);
            window.TrackLastControl("chip");
            window.EndColumn();
            window.Present();
            return activated;
        }
    }

    /// <summary>Applies CUI's default and upper limit when the maximum is outside its normal range.</summary>
    [GuiFact]
    public void RatingUsesTheNativeStarCount()
    {
        using var window = new CuiWindow("Rating", 640, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Rating("rating", 6, 0, true));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Rating("rating", 17, 30, true));
        window.BeginColumn();
        Assert.Equal(-1, window.Rating("default", 5, 0, true));
        window.TrackLastControl("default");
        Assert.Equal(-1, window.Rating("capped", 16, 30, true));
        window.TrackLastControl("capped");
        window.EndColumn();
        window.Present();

        Assert.Equal(5 * 22 + 4 * 2, window.GetControlRectangle("default").Width);
        Assert.Equal(16 * 22 + 15 * 2, window.GetControlRectangle("capped").Width);
    }
}
