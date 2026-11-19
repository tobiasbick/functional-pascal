// [Documentation](../../../docs/ui/gui/text-and-display/canvas.md)
namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks CUI-owned canvas wheel routing, opt-in behavior and retirement.</summary>
public sealed class CuiCanvasWheelTests
{
    /// <summary>Consumes both axes once and stops publishing deltas when disabled or unsubscribed.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void CanvasUsesNativeWheelOwnership(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Canvas wheel", 640, 300, renderer, continuous: true);
        Draw(); Draw();
        Assert.Equal((2f, -3f), Draw(2, -3));
        Assert.Equal((0f, 0f), Draw());
        Assert.Equal((0f, 0f), Draw(2, -3, enabled: false));
        Assert.Equal((0f, 0f), Draw(2, -3, wheel: false));
        Assert.True(window.NextFrame()); window.Text("Removed"); window.Present();
        Draw(); Draw();
        Assert.Equal((1f, 1f), Draw(1, 1));

        (float, float) Draw(float dx = 0, float dy = 0, bool enabled = true, bool wheel = true)
        {
            Assert.True(window.NextFrame()); window.MovePointer(70, 50);
            if (dx != 0 || dy != 0) window.Wheel(dx, dy);
            window.BeginColumn(8, 0);
            window.DrawingCanvas("canvas", 1, 0, 2, 140, [], enabled,
                out _, out _, out _, out _, wheel, out float x, out float y);
            window.TrackLastControl("canvas");
            window.EndColumn(); window.Present();
            return (x, y);
        }
    }

    /// <summary>Lets CUI route scrolling to the enclosing page when there is no active handler.</summary>
    /// <param name="enabled">Whether the canvas accepts interaction.</param>
    /// <param name="wheel">Whether a wheel handler is registered.</param>
    [GuiTheory]
    [InlineData(true, true)]
    [InlineData(true, false)]
    [InlineData(false, true)]
    public void CanvasWheelHandlerOwnsOnlyItsRegion(bool enabled, bool wheel)
    {
        using var window = new CuiWindow("Canvas in scroll", 400, 200, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Draw(); Draw();
        float taken = Draw(-2);
        window.ScrollState("page", -1, 0, out float current, out float maximum);
        Assert.True(maximum > 0);
        if (enabled && wheel)
        {
            Assert.Equal(-2, taken);
            Assert.Equal(0, current);
        }
        else
        {
            Assert.Equal(0, taken);
            Assert.True(current > 0);
        }

        float Draw(float delta = 0)
        {
            Assert.True(window.NextFrame()); window.MovePointer(60, 40);
            window.Wheel(delta);
            window.BeginScroll("page", 120);
            window.DrawingCanvas("canvas", 1, 0, 2, 300, [], enabled,
                out _, out _, out _, out _, wheel, out _, out float taken);
            window.TrackLastControl("canvas");
            window.EndScroll(); window.Present();
            return taken;
        }
    }
}
