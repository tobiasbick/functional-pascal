using System.Buffers.Binary;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Fpas.Gui.Cui.Text;

[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace Fpas.Gui.Cui.Tests;

/// <summary>Exercises the actual native window, input ABI, rendering and UI-thread lifetime.</summary>
public sealed class CuiWindowTests
{
    /// <summary>Routes button, boolean and UTF-8 edits through CUI while managed memory moves.</summary>
    /// <param name="renderer">The renderer required by this integration case.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    [Trait("Category", "NativeGui")]
    public void ControlsRoundTripAcrossTheNativeBoundary(CuiRenderer renderer)
    {
        using var window = new CuiWindow("FPAS CUI binding: Grüße", renderer: renderer, continuous: true);
        Assert.Equal(renderer == CuiRenderer.Software, window.IsSoftware);
        Assert.Equal(16, Marshal.SizeOf<CuiRectangle>());
        window.SetReducedMotion(true);
        var text = new CuiTextBuffer("Grüße");
        int clicks = 0, edits = 0, checks = 0;
        bool enabled = false;
        CuiRectangle button = default, checkbox = default, editor = default;
        for (int frame = 0; frame < 60; frame++)
        {
            Assert.True(window.NextFrame());
            CuiRectangle target = frame switch
            {
                10 or 11 or 12 => button,
                20 or 21 or 22 => checkbox,
                30 or 31 or 32 => editor,
                _ => default
            };
            if (target.Width > 0)
            {
                float x = frame is 30 or 31 or 32 ? target.X + target.Width - 12 : target.X + target.Width / 2;
                window.MovePointer(x, target.Y + target.Height / 2);
            }
            if (frame is 11 or 21 or 31) window.SetLeftButton(true);
            if (frame is 12 or 22 or 32) window.SetLeftButton(false);
            if (frame == 36) window.TypeText(" — schön ✓");
            window.BeginColumn();
            window.Heading("FPAS managed binding");
            if (window.Button("Increment##counter")) clicks++;
            button = window.LastRectangle;
            if (window.Checkbox("Enabled##checkbox", ref enabled)) checks++;
            checkbox = window.LastRectangle;
            if ((window.TextInput("text", text) & CuiTextInputEvents.Changed) != 0) edits++;
            editor = window.LastRectangle;
            window.Text(text.Text);
            window.EndColumn();
            if (frame % 7 == 0) GC.Collect(2, GCCollectionMode.Forced, blocking: true, compacting: true);
        }
        Assert.Equal(1, clicks);
        Assert.True(enabled);
        Assert.Equal(1, checks);
        Assert.Equal(1, edits);
        Assert.Equal("Grüße — schön ✓", text.Text);
        Capture(window, 640, 480);
    }

    /// <summary>Prevents cross-thread use and verifies disposal and layout-state failures.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void OwnershipAndFrameStateAreEnforced()
    {
        using var window = new CuiWindow("Lifetime", renderer: CuiRenderer.Software, continuous: true);
        Assert.Throws<InvalidOperationException>(() => window.Text("Outside a frame"));
        Exception? failure = null;
        var worker = new Thread(() =>
        {
            try
            {
                Assert.Throws<InvalidOperationException>(() => window.NextFrame());
                Assert.Throws<InvalidOperationException>(() => window.Dispose());
            }
            catch (Exception error) { failure = error; }
        });
        worker.Start();
        Assert.True(worker.Join(TimeSpan.FromSeconds(5)));
        Assert.Null(failure);
        Assert.True(window.NextFrame());
        Assert.Throws<InvalidOperationException>(() => window.EndColumn());
        window.BeginColumn();
        Assert.Throws<InvalidOperationException>(() => window.NextFrame());
        window.EndColumn();
        Assert.Throws<InvalidOperationException>(() => window.CapturePng("unused.png"));
        window.Dispose();
        window.Dispose();
        Assert.Throws<ObjectDisposedException>(() => window.NextFrame());
    }

    /// <summary>Captures both growing and shrinking frames using the correct pixel-buffer bounds.</summary>
    /// <param name="renderer">The renderer whose pixel-stride path is tested.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    [Trait("Category", "NativeGui")]
    public void CaptureSurvivesViewportChanges(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Resize capture", renderer: renderer, continuous: true);
        Assert.Equal(renderer == CuiRenderer.Software, window.IsSoftware);
        window.SetReducedMotion(true);
        Draw(window);
        Draw(window);
        foreach (var (width, height) in new[] { (800, 600), (480, 360) })
        {
            window.Resize(width, height);
            var deadline = Stopwatch.StartNew();
            do { Draw(window); }
            while ((window.Size.Width != width || window.Size.Height != height) &&
                deadline.Elapsed < TimeSpan.FromSeconds(5));
            Assert.Equal(width, window.Size.Width);
            Assert.Equal(height, window.Size.Height);
            Capture(window, width, height);
        }
    }

    /// <summary>A failed PNG write still closes the native render frame and permits recovery.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void CaptureFailureRequiresANewFrameAndAllowsRecovery()
    {
        using var window = new CuiWindow("Capture failure", renderer: CuiRenderer.Software, continuous: true);
        Draw(window);
        Draw(window);
        string path = Path.Combine(Path.GetTempPath(), $"fpas-cui-missing-{Guid.NewGuid():N}", "capture.png");
        Assert.Throws<IOException>(() => window.CapturePng(path));
        Assert.Throws<InvalidOperationException>(() => window.Text("Frame already rendered"));
        Draw(window);
        Capture(window, 640, 480);
    }

    private static void Draw(CuiWindow window)
    {
        Assert.True(window.NextFrame());
        window.BeginColumn();
        window.Heading("Owned native window");
        window.EndColumn();
    }

    private static void Capture(CuiWindow window, int width, int height)
    {
        string path = Path.Combine(Path.GetTempPath(), $"fpas-cui-{Guid.NewGuid():N}.png");
        try
        {
            window.CapturePng(path);
            byte[] image = File.ReadAllBytes(path);
            Assert.Equal(new byte[] { 137, 80, 78, 71, 13, 10, 26, 10 }, image[..8]);
            Assert.Equal(width, BinaryPrimitives.ReadInt32BigEndian(image.AsSpan(16, 4)));
            Assert.Equal(height, BinaryPrimitives.ReadInt32BigEndian(image.AsSpan(20, 4)));
        }
        finally { File.Delete(path); }
    }
}
