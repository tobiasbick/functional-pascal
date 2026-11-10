using System.Buffers.Binary;
using System.Diagnostics;
using Fpas.Tests.Support;

[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises the actual native window, input ABI, rendering and UI-thread lifetime.</summary>
public sealed class CuiWindowTests
{
    /// <summary>Keeps focus when an icon changes and suppresses activation while busy or disabled.</summary>
    [GuiFact]
    public void IconButtonKeepsItsIdentityAndHonorsInputState()
    {
        using var window = new CuiWindow("Icon button", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.False(Draw("\uE115", "Menu", focus: true));
        var bounds = window.GetControlRectangle("icon");
        Assert.True(bounds.Width > 0);
        Assert.Equal(bounds.Width, bounds.Height);

        Assert.True(Draw("\uE13D", "Add", key: 2));
        Assert.True(window.ControlFocused("icon"));
        Assert.False(Draw("\uE13D", "Adding", key: 6, loading: true));
        Assert.True(window.ControlFocused("icon"));
        Assert.False(Draw("\uE13D", "Add", key: 2, enabled: false));
        Assert.False(window.ControlFocused("icon"));

        bool Draw(string glyph, string title, int key = 0, bool enabled = true, bool loading = false, bool focus = false)
        {
            Assert.True(window.NextFrame());
            if (key != 0)
            {
                window.SetKey(key, true);
                window.SetKey(key, false);
            }
            window.BeginColumn();
            bool activated = window.IconButton("icon", glyph, title, "", 6, 0, enabled, loading, false, focus);
            window.TrackLastControl("icon");
            window.EndColumn();
            window.Present();
            return activated;
        }
    }

    /// <summary>Places row children side by side with the requested gap inside a column.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void RowsPlaceChildrenBesideEachOtherInsideAColumn()
    {
        using var window = new CuiWindow("Nested layout", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        const float gap = 16;
        CuiRectangle first = default, second = default, below = default;
        for (int frame = 0; frame < 3; frame++)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn();
            window.BeginRow(padding: 6, gap: gap);
            window.Button("First");
            first = window.LastRectangle;
            window.Button("Second");
            second = window.LastRectangle;
            window.EndRow();
            window.Button("Below");
            below = window.LastRectangle;
            window.EndColumn();
        }
        Assert.True(first.Width > 0 && second.Width > 0);
        Assert.InRange(MathF.Abs(first.Y - second.Y), 0, 0.01f);
        Assert.InRange(second.X - (first.X + first.Width), gap - 0.01f, gap + 0.01f);
        Assert.True(below.Y >= second.Y + second.Height);
    }

    /// <summary>Prevents cross-thread use and verifies disposal and layout-state failures.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void OwnershipAndFrameStateAreEnforced()
    {
        using var window = new CuiWindow("Lifetime", renderer: CuiRenderer.Software, continuous: true);
        Assert.Throws<InvalidOperationException>(() => window.Text("Outside a frame"));
        Assert.Throws<InvalidOperationException>(() => window.RequestFocus("outside"));
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
        window.BeginRow();
        Assert.Throws<InvalidOperationException>(() => window.EndColumn());
        Assert.Throws<InvalidOperationException>(() => window.NextFrame());
        window.EndRow();
        Assert.Throws<InvalidOperationException>(() => window.EndRow());
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
        using var directory = new TemporaryDirectory();
        string path = directory.File("capture-雪.png");
        window.CapturePng(path);
        byte[] image = File.ReadAllBytes(path);
        Assert.Equal(new byte[] { 137, 80, 78, 71, 13, 10, 26, 10 }, image[..8]);
        Assert.Equal(width, BinaryPrimitives.ReadInt32BigEndian(image.AsSpan(16, 4)));
        Assert.Equal(height, BinaryPrimitives.ReadInt32BigEndian(image.AsSpan(20, 4)));
    }
}
