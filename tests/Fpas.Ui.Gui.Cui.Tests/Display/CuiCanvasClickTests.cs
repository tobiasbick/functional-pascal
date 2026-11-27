// [Documentation](../../../docs/ui/gui/text-and-display/canvas.md)
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks that native secondary activation follows the canvas hit and input state.</summary>
public sealed class CuiCanvasClickTests
{
    /// <summary>Routes one secondary event without activating the primary action.</summary>
    /// <param name="renderer">Native renderer used to lay out and hit-test both canvases.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void RightClickUsesOnlyTheTopmostEnabledCanvas(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Canvas clicks", 380, 280, renderer, continuous: true);
        var rightClicks = new List<(string Id, CuiCanvasClick Click)>();
        var primaryClicks = new List<string>();
        CuiRectangle box = default;

        Frame();
        box = window.GetControlRectangle("back");
        Assert.True(box.Width > 20 && box.Height > 15);
        Frame(right: true);
        var first = Assert.Single(rightClicks);
        Assert.Equal("back", first.Id);
        Assert.Equal(new CuiCanvasClick(20, 15, box.Width, box.Height), first.Click);
        Frame();
        Frame(right: false);
        Assert.Single(rightClicks);
        Assert.Empty(primaryClicks);

        Frame(left: true);
        Frame(left: false);
        Assert.Equal("back", Assert.Single(primaryClicks));
        Assert.Single(rightClicks);

        Frame(enabled: false);
        Frame(enabled: false, right: true);
        Frame(enabled: false, right: false);
        Assert.Single(rightClicks);

        Frame(cover: true);
        Frame(cover: true, right: true);
        Frame(cover: true, right: false);
        Assert.Equal(new[] { "back", "front" }, rightClicks.Select(value => value.Id));
        Assert.Single(primaryClicks);

        void Frame(bool enabled = true, bool cover = false, bool? left = null, bool? right = null)
        {
            Assert.True(window.NextFrame());
            if (left.HasValue || right.HasValue) window.MovePointer(box.X + 20, box.Y + 15);
            if (left.HasValue) window.SetLeftButton(left.Value);
            if (right.HasValue) window.SetRightButton(right.Value);
            window.BeginLayers();
            Draw("back", enabled);
            if (cover) Draw("front", enabled: true);
            window.EndGroup();
            window.Present();
        }

        void Draw(string id, bool enabled)
        {
            if (window.DrawingCanvas(id, 2, 180, 2, 100, [], enabled, out _, out _, out _, out _,
                false, out _, out _, onRightClick: click => rightClicks.Add((id, click))))
                primaryClicks.Add(id);
            window.TrackLastControl(id);
        }
    }
}
