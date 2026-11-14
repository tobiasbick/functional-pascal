using Fpas.Ui.Gui.Cui.Widgets.Actions;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks explicit presentation, finalized first-frame geometry and native close.</summary>
public sealed class CuiPresentationTests
{
    /// <summary>Implicit presentation retires controls and modal scopes omitted from the view.</summary>
    [GuiFact]
    public void ImplicitPresentationRetiresRemovedState()
    {
        using var window = new CuiWindow("Implicit retirement", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.True(window.NextFrame());
        DrawConfirm();
        Assert.True(window.NextFrame());
        window.SetKey(2, true); window.SetKey(2, false);
        Assert.True(DrawConfirm().Armed);
        Assert.True(window.NextFrame());
        Assert.True(DrawConfirm().Armed);
        int sheet = window.BeginSheet("sheet", "Title", "", 0, 160, true, true, false, "close");
        Assert.NotEqual(0, sheet & 1);
        window.EndSheet();
        Assert.True(window.NextFrame());
        window.Text("Control and sheet removed");
        Assert.True(window.NextFrame());
        Assert.False(DrawConfirm().Armed);
        // A retired sheet has no outgoing animation when it is reintroduced closed.
        // Without retirement, the previous open sheet still has a visible spring.
        window.SetReducedMotion(false);
        Assert.Equal(0, window.BeginSheet("sheet", "Title", "", 0, 160, false, true, false, "close") & 1);

        CuiConfirmButtonResult DrawConfirm()
        {
            var result = window.ConfirmButton("confirm", "Delete", "Confirm", "", 30000, 2, 0, true, true);
            window.TrackLastControl("confirm");
            return result;
        }
    }

    /// <summary>Publishes a frame without an input wait and bounds inspection to that frame.</summary>
    /// <param name="renderer">The renderer whose present path is exercised.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void PresentationFinalizesGeometryAndClosesTheFrame(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Explicit presentation", renderer: renderer, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginColumn();
        window.Button("Track###button");
        window.TrackLastControl("button");
        Assert.Throws<InvalidOperationException>(() => window.Present());
        Assert.Throws<InvalidOperationException>(() => window.GetControlRectangle("button"));
        window.EndColumn();
        window.Present();
        var bounds = window.GetControlRectangle("button");
        Assert.True(bounds.Width > 0 && bounds.Height > 0);
        Assert.Throws<InvalidOperationException>(() => window.Text("Closed frame"));
        Assert.True(window.NextFrame());
        window.Text("New frame");
        window.Present();
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("button"));
        window.RequestClose();
        Assert.False(window.NextFrame());
        Assert.False(window.TryWake());
    }
}
