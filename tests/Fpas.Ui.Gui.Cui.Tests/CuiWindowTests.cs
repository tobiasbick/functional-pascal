using System.Buffers.Binary;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Text;
using Fpas.Tests.Support;

[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace Fpas.Ui.Gui.Cui.Tests;

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
            // Each press moves from outside the controls after the native frame has begun.
            if (frame is 10 or 20 or 30) window.MovePointer(-1, -1);
            CuiRectangle target = frame switch
            {
                11 or 12 => button,
                21 or 22 => checkbox,
                31 or 32 => editor,
                _ => default
            };
            if (target.Width > 0)
            {
                float x = frame is 31 or 32 ? target.X + target.Width - 12 : target.X + target.Width / 2;
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

    /// <summary>Moves text input to the requested UTF-8 control identity without pointer input.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void RequestFocusRoutesTextToTheRequestedEditor()
    {
        using var window = new CuiWindow("Programmatic focus", renderer: CuiRenderer.Software, continuous: true);
        var first = new CuiTextBuffer("");
        var second = new CuiTextBuffer("");
        const string firstId = "first";
        const string secondId = "Ziel-🧭";

        Assert.True(window.NextFrame());
        window.RequestFocus(firstId);
        Assert.True(window.TextInput(firstId, first).HasFlag(CuiTextInputEvents.Focused));
        window.TextInput(secondId, second);

        Assert.True(window.NextFrame());
        window.TypeText("eins");
        window.TextInput(firstId, first);
        window.RequestFocus(secondId);
        Assert.True(window.TextInput(secondId, second).HasFlag(CuiTextInputEvents.Focused));

        Assert.True(window.NextFrame());
        window.TypeText("Grüße");
        window.TextInput(firstId, first);
        window.TextInput(secondId, second);
        Assert.Equal("eins", first.Text);
        Assert.Equal("Grüße", second.Text);
    }

    /// <summary>Preserves Unicode and line breaks when editing a native text area.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void TextAreaEditsMultilineTextAndUsesRequestedHeight()
    {
        using var window = new CuiWindow("Multiline input", renderer: CuiRenderer.Software, continuous: true);
        var text = new CuiTextBuffer("\n東京 🧭");
        var other = new CuiTextBuffer("");
        const string id = "document";
        const float height = 136;

        Assert.True(window.NextFrame());
        window.RequestFocus(id);
        window.TextArea(id, text, height);
        window.TextInput("other", other);

        Assert.True(window.NextFrame());
        window.TypeText("Grüße");
        var events = window.TextArea(id, text, height);
        Assert.Equal(height, window.LastRectangle.Height);
        window.RequestFocus("other");
        window.TextInput("other", other);

        Assert.True(events.HasFlag(CuiTextInputEvents.Changed));
        Assert.Equal("Grüße\n東京 🧭", text.Text);
        Assert.Empty(other.Text);
    }

    /// <summary>Identifies the control exercised by the controlled selection cases.</summary>
    public enum SelectionWidget
    {
        /// <summary>A list of selectable items.</summary>
        List,
        /// <summary>A table of selectable rows.</summary>
        Table,
        /// <summary>A horizontal row of tab items.</summary>
        Tabs
    }

    /// <summary>Reports activation of the matching item and paints the selection supplied by the host.</summary>
    /// <param name="widget">The kind of control to render and activate.</param>
    [GuiTheory]
    [InlineData(SelectionWidget.List)]
    [InlineData(SelectionWidget.Table)]
    [InlineData(SelectionWidget.Tabs)]
    [Trait("Category", "NativeGui")]
    public void SelectionWidgetsReportActivationAndUseHostSelection(SelectionWidget widget)
    {
        const int width = 640, height = 480;
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Controlled selection", width, height, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        int firstActivations = 0, secondActivations = 0;
        CuiRectangle first = default, second = default;

        DrawWithoutHover(secondSelected: false);
        var initial = CaptureSelection("initial.png");
        Assert.NotEqual(initial.First, initial.Second);

        for (int frame = 0; frame < 3; frame++)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(second.X + second.Width / 2, second.Y + second.Height / 2);
            if (frame > 0) window.SetLeftButton(frame == 1);
            DrawItems(secondSelected: false);
        }

        DrawWithoutHover(secondSelected: false);
        Assert.Equal(initial, CaptureSelection("after-click.png"));

        DrawWithoutHover(secondSelected: true);
        Assert.Equal((initial.Second, initial.First), CaptureSelection("host-selected.png"));
        Assert.Equal((0, 1), (firstActivations, secondActivations));

        void DrawWithoutHover(bool secondSelected)
        {
            for (int frame = 0; frame < 3; frame++)
            {
                Assert.True(window.NextFrame());
                window.MovePointer(4, 4);
                DrawItems(secondSelected);
            }
        }

        void DrawItems(bool secondSelected)
        {
            Action close;
            switch (widget)
            {
                case SelectionWidget.List:
                    window.BeginColumn();
                    close = window.EndColumn;
                    break;
                case SelectionWidget.Table:
                    window.BeginTable("Auswahl-🧭", ["Einträge", "Status"]);
                    close = window.EndTable;
                    break;
                default:
                    window.BeginRow();
                    close = window.EndRow;
                    break;
            }
            first = DrawItem("Grüße###first", !secondSelected, ref firstActivations);
            second = DrawItem("Grüße###second", secondSelected, ref secondActivations);
            close();
        }

        CuiRectangle DrawItem(string label, bool selected, ref int activations)
        {
            bool activated = widget switch
            {
                SelectionWidget.List => window.ListItem(label, selected),
                SelectionWidget.Table => window.TableRow(selected),
                _ => window.TabItem(label, selected)
            };
            if (activated) activations++;
            CuiRectangle bounds = window.LastRectangle;
            if (widget == SelectionWidget.Table)
            {
                window.TableCell("Grüße");
                window.TableCell("Bereit");
            }
            return bounds;
        }

        (uint First, uint Second) CaptureSelection(string filename)
        {
            string path = directory.File(filename);
            window.CapturePng(path);
            uint[] pixels = PngAssertions.ReadPixels(path, width, height);
            return (pixels[BackgroundIndex(first)], pixels[BackgroundIndex(second)]);
        }

        static int BackgroundIndex(CuiRectangle item)
        {
            Assert.True(item.Width > 0 && item.Height > 0);
            int x = (int)(item.X + item.Width * 0.75f);
            int y = (int)(item.Y + item.Height * 0.8f);
            return y * width + x;
        }
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
        Assert.Throws<ArgumentException>(() => window.BeginTable("table", []));
        window.BeginTable("table", ["Column"]);
        Assert.Throws<InvalidOperationException>(() => window.TableCell("No row"));
        Assert.Throws<InvalidOperationException>(() => window.EndColumn());
        Assert.Throws<InvalidOperationException>(() => window.NextFrame());
        window.TableRow(selected: false);
        window.TableCell("Cell");
        Assert.Throws<InvalidOperationException>(() => window.TableCell("Extra cell"));
        window.TableRow(selected: false);
        window.TableCell("Next row");
        window.EndTable();
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
