using Fpas.Tests.Support;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Exercises texture uploads, replacement and ownership through real native renderers.</summary>
public sealed class CuiImageTests
{
    /// <summary>Preserves a queued image through disposal and a replacement upload in the same frame.</summary>
    /// <param name="renderer">The renderer required by this integration case.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    [Trait("Category", "NativeGui")]
    public void QueuedImageRendersAfterDisposalAndReplacement(CuiRenderer renderer)
    {
        const int width = 360, height = 240;
        using var window = new CuiWindow("Image replacement", width, height, renderer, continuous: true);
        using var directory = new TemporaryDirectory();
        Assert.Equal(renderer == CuiRenderer.Software, window.IsSoftware);
        byte[] rgba = [255, 0, 0, 255];
        using var original = window.CreateImage(1, 1, rgba);
        rgba[0] = 0;
        rgba[2] = 255;

        Assert.True(window.NextFrame());
        Assert.True(window.NextFrame());
        window.BeginColumn(padding: 12, gap: 4);
        window.Image(original, 40, 40);
        original.Dispose();
        Assert.Throws<ObjectDisposedException>(() => window.Image(original, 40, 40));
        using var replacement = window.CreateImage(1, 1, rgba);
        window.Image(replacement, 40, 40);
        window.EndColumn();
        string both = directory.File("both.png");
        window.CapturePng(both);
        uint[] pixels = PngAssertions.ReadPixels(both, width, height);
        Assert.Equal(0xff0000ffu, pixels[24 * width + 24]);
        Assert.Equal(0xffff0000u, pixels[68 * width + 24]);

        // Present a frame that no longer references the original, then draw again after retirement.
        Assert.True(window.NextFrame());
        Assert.True(window.NextFrame());
        window.BeginColumn(padding: 12);
        window.Image(replacement, 40, 40);
        window.EndColumn();
        string next = directory.File("replacement.png");
        window.CapturePng(next);
        Assert.Equal(0xffff0000u, PngAssertions.ReadPixels(next, width, height)[24 * width + 24]);
    }

    /// <summary>Rejects invalid pixel buffers and enforces window and thread ownership until teardown.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void TextureStaysWithItsWindowAndUiThread()
    {
        using var owner = new CuiWindow("Image owner", renderer: CuiRenderer.OpenGl, continuous: true);
        Assert.False(owner.IsSoftware);
        Assert.Throws<InvalidOperationException>(() =>
        {
            using var conflict = new CuiWindow("Second OpenGL window", renderer: CuiRenderer.OpenGl);
        });
        using var other = new CuiWindow("Other window", continuous: true);
        Assert.True(other.IsSoftware);
        Assert.Throws<ArgumentException>(() => owner.CreateImage(2, 2, [255, 0, 0, 255]));
        using var image = owner.CreateImage(1, 1, [255, 0, 0, 255]);
        Assert.True(other.NextFrame());
        Assert.Throws<ArgumentException>(() => other.Image(image, 40, 40));

        Exception? failure = null;
        var worker = new Thread(() => failure = Record.Exception(image.Dispose));
        worker.Start();
        Assert.True(worker.Join(TimeSpan.FromSeconds(5)));
        Assert.IsType<InvalidOperationException>(failure);
        Assert.True(owner.NextFrame());
        owner.Image(image, 40, 40);
        owner.Dispose();
        image.Dispose();
    }

    /// <summary>Checks native texture counts through failed uploads, frame retirement and repeated window teardown.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void NativeTextureCountsReturnToZeroAcrossWindowSessions()
    {
        using var audit = new CuiTextureAudit();
        byte[] pixels = [255, 0, 0, 255];
        const int sessions = 3;
        for (int session = 0; session < sessions; session++)
        {
            using (var window = new CuiWindow("Texture accounting", renderer: CuiRenderer.Software, continuous: true))
            {
                audit.Attach(window);
                using (window.CreateImage(1, 1, pixels))
                    Assert.Equal(1, audit.ImageCount);
                Assert.Equal(0, audit.ImageCount);

                var original = window.CreateImage(1, 1, pixels);
                audit.RejectNextImage();
                Assert.Throws<InvalidOperationException>(() => window.CreateImage(1, 1, pixels));
                Assert.Equal(1, audit.ImageCount);

                Assert.True(window.NextFrame());
                window.Image(original, 40, 40);
                original.Dispose();
                var replacement = window.CreateImage(1, 1, pixels);
                Assert.Equal(2, audit.ImageCount);
                Assert.True(window.NextFrame());
                Assert.Equal(2, audit.ImageCount);
                window.Image(replacement, 40, 40);
                Assert.True(window.NextFrame());
                Assert.Equal(1, audit.ImageCount);

                var queued = window.CreateImage(1, 1, pixels);
                window.Image(queued, 40, 40);
                queued.Dispose();
                Assert.Equal(2, audit.ImageCount);
            }
            Assert.Null(audit.CallbackFailure);
            Assert.Equal(0, audit.TextureCount);
        }
        Assert.Equal(sessions, audit.RejectedUploads);
    }
}
