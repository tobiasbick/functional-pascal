using Fpas.Gui.Cui.Text;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Checks declarative resource reconciliation against native allocation accounting.</summary>
public sealed class CuiHostResourceTests
{
    /// <summary>Leaves room for further native typing after the model accepts a nearly full buffer.</summary>
    [GuiFact]
    public void AcceptedDocumentsGrowTheirEditingCapacity()
    {
        using var window = new CuiWindow("Editor growth", renderer: CuiRenderer.Software, continuous: true);
        using var resources = new CuiHostResources(window);
        var first = resources.Editor("editor", "", default, false);
        int capacity = first.Capacity;
        string accepted = new('x', capacity - 1);
        var next = resources.Editor("editor", accepted, default, false);
        Assert.Same(first, next);
        Assert.True(next.Capacity >= capacity * 2);
        Assert.Equal(accepted, next.Text);
        Assert.Same(next, resources.Editor("editor", accepted, default, false));
        Assert.Equal(1, resources.EditorCount);
    }
    /// <summary>Releases removed editors and replaced textures through repeated host sessions.</summary>
    [GuiFact]
    public void RemovalReplacementAndCloseReleaseOwnedNativeResources()
    {
        using var audit = new CuiTextureAudit();
        for (int run = 0; run < 3; run++)
        {
            using (var window = new CuiWindow("Host resources", renderer: CuiRenderer.Software, continuous: true))
            {
                audit.Attach(window);
                using var resources = new CuiHostResources(window);
                byte[] pixels = [255, 0, 0, 255];
                for (int frame = 0; frame < 6; frame++)
                {
                    Assert.True(window.NextFrame());
                    resources.BeginFrame();
                    if (frame < 2)
                    {
                        var editor = resources.Editor("editor", frame == 0 ? "λ👋" : "new", default, false);
                        window.Editor("editor", editor, false, true, true, "Hint");
                        window.TrackLastControl("editor");
                        window.Image(resources.Image("image", 1, 1, frame == 0 ? pixels : [0, 0, 255, 255]), 32, 32);
                    }
                    window.Present();
                    resources.EndFrame();
                    Assert.Equal(frame < 2 ? 1 : 0, resources.EditorCount);
                    Assert.Equal(frame < 2 ? 1 : 0, resources.ImageCount);
                }
                Assert.Equal(0, audit.ImageCount);
            }
            Assert.Null(audit.CallbackFailure);
            Assert.Equal(0, audit.TextureCount);
        }
    }

    /// <summary>Computes selection offsets after normalizing pasted line endings.</summary>
    [Fact]
    public void EditorNormalizationKeepsDirectionalUnicodeSelection()
    {
        var buffer = new CuiTextBuffer("λ\r\n👋\rZ");
        buffer.Selection = new(7, 3);
        CuiHostResources.NormalizeEditor(buffer, multiline: true);
        Assert.Equal("λ\n👋\nZ", buffer.Text);
        Assert.Equal(new CuiTextSelection(6, 2), buffer.Selection);
        CuiHostResources.NormalizeEditor(buffer, multiline: false);
        Assert.Equal("λ 👋 Z", buffer.Text);
        Assert.Equal(new CuiTextSelection(6, 2), buffer.Selection);
    }
}
