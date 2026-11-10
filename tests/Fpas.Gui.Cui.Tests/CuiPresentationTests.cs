namespace Fpas.Gui.Cui.Tests;

/// <summary>Checks explicit presentation, finalized first-frame geometry and native close.</summary>
public sealed class CuiPresentationTests
{
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
