// [Documentation](../../../docs/ui/gui/app-chrome/window.md)
using System.Diagnostics;
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Desktop;

[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Exercises the actual native window, input ABI, rendering and UI-thread lifetime.</summary>
public sealed class CuiWindowTests
{
    /// <summary>Checks stable toggle focus and previous-frame row geometry when the caption is hidden.</summary>
    /// <param name="checkbox">Whether to use a checkbox instead of a switch.</param>
    [GuiTheory]
    [InlineData(true)]
    [InlineData(false)]
    public void ToggleWithoutCaptionKeepsNativeRowIdentity(bool checkbox)
    {
        using var window = new CuiWindow("Toggle row", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        var first = Draw(hideLabel: false, focus: true);
        Assert.False(first.Changed);
        Assert.Equal(default(CuiRectangle), first.Previous);
        Assert.True(window.ControlFocused("toggle"));
        var labeled = window.GetControlRectangle("toggle");

        var blankDescription = Draw(hideLabel: false, description: "");
        Assert.Equal(labeled, blankDescription.Previous);
        var described = window.GetControlRectangle("toggle");
        Assert.True(described.Height > labeled.Height);

        var hidden = Draw(hideLabel: true, key: 6);
        Assert.True(hidden.Changed);
        Assert.Equal(described, hidden.Previous);
        Assert.True(window.ControlFocused("toggle"));
        var track = window.GetControlRectangle("toggle");
        Assert.Equal(checkbox ? 18f : 36f, track.Width);
        Assert.True(labeled.Width > track.Width);

        var disabled = Draw(hideLabel: false, key: 2, enabled: false);
        Assert.False(disabled.Changed);
        Assert.Equal(track, disabled.Previous);
        Assert.False(window.ControlFocused("toggle"));

        (bool Changed, CuiRectangle Previous) Draw(bool hideLabel, int key = 0,
            bool enabled = true, bool focus = false, string? description = null)
        {
            Assert.True(window.NextFrame());
            if (key != 0)
            {
                window.SetKey(key, true);
                window.SetKey(key, false);
            }
            window.BeginColumn();
            bool changed = checkbox
                ? window.Checkbox("toggle", "Notifications", description, true, enabled, focus, indeterminate: true, hideLabel)
                : window.Switch("toggle", "Notifications", description, true, enabled, focus, hideLabel);
            var previous = window.LastRectangle;
            window.TrackLastControl("toggle");
            window.EndColumn();
            window.Present();
            return (changed, previous);
        }
    }

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

    /// <summary>Closing a row and its enclosing column makes each box the target of subsequent sizing and tracking.</summary>
    [GuiFact]
    public void ClosingNestedBoxesMakesTheClosedBoxLast()
    {
        using var window = new CuiWindow("Nested boxes", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginColumn(0, 0);
        window.BeginRow(0, 0);
        window.Skeleton("child", 2, 20, 12);
        window.TrackLastControl("child");
        window.EndRow();
        window.SizeLast(false, 2, 44, 0, -1, 0, 0);
        window.TrackLastControl("row");
        window.Skeleton("sibling", 2, 30, 20);
        window.TrackLastControl("sibling");
        window.EndColumn();
        window.SizeLast(true, 2, 180, 0, -1, 0, 0);
        window.TrackLastControl("column");
        window.Present();

        var row = window.GetControlRectangle("row");
        var column = window.GetControlRectangle("column");
        var sibling = window.GetControlRectangle("sibling");
        Assert.Equal(12, window.GetControlRectangle("child").Height);
        Assert.Equal((180f, 44f), (row.Width, row.Height));
        Assert.Equal((180f, 64f), (column.Width, column.Height));
        Assert.Equal(30, sibling.Width);
        Assert.Equal(row.Y + row.Height, sibling.Y);
    }

    /// <summary>Rejecting a box at CUI's capacity leaves the existing boxes open and closable.</summary>
    [GuiFact]
    public void BoxNestingLimitLeavesOpenLayoutsBalanced()
    {
        using var window = new CuiWindow("Box capacity", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        int opened = 0;
        for (; opened < 1024; opened++)
        {
            var failure = Record.Exception(() => window.BeginColumn(0, 0));
            if (failure is null) continue;
            Assert.Contains("nesting limit", Assert.IsType<InvalidOperationException>(failure).Message);
            break;
        }
        Assert.InRange(opened, 1, 1023);
        Assert.Throws<InvalidOperationException>(() => window.BeginGrid(1, 0, 0, 0));
        Assert.Throws<InvalidOperationException>(() => window.BeginScroll("overflow"));
        Assert.Throws<InvalidOperationException>(() => window.BeginViewport("overflow", true));

        window.EndColumn();
        var split = new CuiSplitViewOptions(2, 100, 2, 100, false, 0, 0);
        Assert.Throws<InvalidOperationException>(() => window.BeginSplitView("overflow", 0.5f, split, true));
        window.BeginRow(0, 0);
        window.EndRow();
        for (int remaining = opened - 1; remaining > 0; remaining--) window.EndColumn();
        window.Skeleton("after", 2, 80, 20);
        window.TrackLastControl("after");
        window.Present();
        Assert.Equal(80, window.GetControlRectangle("after").Width);
    }

    /// <summary>A split rejects incorrect pane transitions and preserves the two pane layouts.</summary>
    [GuiFact]
    public void SplitPaneTransitionsKeepNativeLayoutBalanced()
    {
        using var window = new CuiWindow("Split layout", 400, 240, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var options = new CuiSplitViewOptions(2, 200, 2, 121, true, 80, 160);
        Assert.Throws<InvalidOperationException>(() => window.NextSplitPane(true));
        Assert.Equal(0.5f, window.BeginSplitView("split", 0.5f, options, true));
        Assert.Throws<InvalidOperationException>(window.EndSplitView);
        Assert.Throws<InvalidOperationException>(window.Present);

        window.BeginColumn(0, 0);
        Assert.Throws<InvalidOperationException>(() => window.NextSplitPane(true));
        window.Skeleton("first", 2, 40, 16);
        window.TrackLastControl("first");
        window.EndColumn();
        window.NextSplitPane(true);
        Assert.Throws<InvalidOperationException>(() => window.NextSplitPane(true));
        window.Skeleton("second", 2, 40, 16);
        window.TrackLastControl("second");
        window.EndSplitView();
        window.TrackLastControl("split");
        window.Present();

        var split = window.GetControlRectangle("split");
        var first = window.GetControlRectangle("first");
        var second = window.GetControlRectangle("second");
        Assert.Equal((200f, 121f), (split.Width, split.Height));
        Assert.Equal(split.Y, first.Y);
        Assert.Equal(split.Y + 41, second.Y, precision: 1);
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
        Assert.Equal(640, window.ViewportSize.Width);
        Assert.Equal(480, window.ViewportSize.Height);
        foreach (var (width, height) in new[] { (800, 600), (480, 360) })
        {
            var before = window.ViewportSize;
            window.Resize(width, height);
            Assert.Equal(before, window.ViewportSize);
            var deadline = Stopwatch.StartNew();
            do { Draw(window); }
            while ((window.ViewportSize.Width != width || window.ViewportSize.Height != height) &&
                deadline.Elapsed < TimeSpan.FromSeconds(5));
            Assert.Equal(width, window.ViewportSize.Width);
            Assert.Equal(height, window.ViewportSize.Height);
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
        PngAssertions.ReadPixels(path, width, height);
    }
}
