// [Documentation](../../../docs/ui/gui/canvas-and-bars/painting.md)
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using System.Runtime.CompilerServices;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks scoped native drawing, callback failures and pointer proposals.</summary>
public sealed class CuiCanvasPaintingTests
{
    /// <summary>Draws after layout, preserves captured data through GC and invalidates the painter on return.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void PaintingUsesFinalGeometryAndNativeResources(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Native painting", 440, 260, renderer, continuous: true);
        window.SetReducedMotion(true);
        CuiCanvasPainter? retained = null;
        bool image = true;
        int paints = 0;
        byte[] red = [255, 0, 0, 255];
        Draw(); Draw();
        Assert.True(paints >= 2);
        Assert.Throws<InvalidOperationException>(() => retained!.Circle(0, 0, 1, uint.MaxValue));
        using var directory = new TemporaryDirectory();
        window.CapturePng(directory.File("red.png"));
        image = false;
        Draw();
        window.CapturePng(directory.File("without.png"));
        Assert.NotEqual(File.ReadAllBytes(directory.File("red.png")), File.ReadAllBytes(directory.File("without.png")));
        Draw(enabled: false);
        Assert.True(window.NextFrame()); window.Text("Removed"); window.Present();
        Draw();

        void Draw(bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(8, 0);
            window.DrawingCanvas("canvas", 1, 0, 2, 180, [], enabled,
                out _, out _, out _, out _, false, out _, out _, painter =>
                {
                    retained = painter; paints++;
                    Assert.Equal(424f, painter.Width);
                    Assert.Equal(180f, painter.Height);
                    Assert.Equal(!enabled, painter.Disabled);
                    Assert.True(painter.Time >= 0 && painter.DeltaTime >= 0);
                    painter.Gradient(0, 0, painter.Width, painter.Height, 8, 0xffeeeeee, 0xffcccccc);
                    painter.Border(5, 5, 200, 90, 5, 0xffeeeeee, 0xff222222, 1);
                    painter.Shadow(20, 25, 40, 40, 5, 6, -1, 0, 2, 0x40222222);
                    painter.Rectangle(20, 25, 40, 40, 5, 0xff229944);
                    painter.Circle(85, 45, 16, 0xffaa6622);
                    painter.Line(20, 80, 170, 80, 2, 0xff222222);
                    painter.Polyline([new(125, 20), new(145, 50), new(170, 25)], 2, 0xff222222);
                    painter.Polygon([new(190, 20), new(215, 50), new(175, 55)], 0xff229944);
                    painter.Arc(245, 40, 18, 0, MathF.PI, 2, 0xffaa6622);
                    Assert.True(painter.Measure("Grüße 日本 🎵", 0, 13).Width > 0);
                    painter.Text(10, 110, "Grüße 日本 🎵", 0, 13, 0xff222222);
                    painter.Icon(280, 25, 24, 24, "\uE13D", 20, 0xff222222);
                    painter.Emoji(320, 25, 24, 24, "🎵");
                    painter.PushClip(0, 140, 120, 40);
                    painter.Circle(80, 150, 30, 0xff229944);
                    if (image) painter.Image(new(null, 1, 1, red), 20, 145, 30, 30, 0, uint.MaxValue);
                    // The callback's remaining clip is unwound before the next widget paints.
                    Assert.IsType<InvalidOperationException>(Task.Factory.StartNew(
                        () => Record.Exception(() => _ = painter.Width), CancellationToken.None,
                        TaskCreationOptions.LongRunning, TaskScheduler.Default).GetAwaiter().GetResult());
                });
            window.Text("Below canvas"); window.EndColumn();
            GC.Collect(); GC.WaitForPendingFinalizers(); window.Present();
        }
    }

    /// <summary>Unwinds native clips and exposes the original exception without permitting window reentry.</summary>
    [GuiFact]
    public void CallbackErrorsReturnThroughManagedPresentation()
    {
        using var window = new CuiWindow("Painting failures", renderer: CuiRenderer.Software, continuous: true);
        var original = new FormatException("Paint failed");
        Canvas(p => { p.PushClip(0, 0, 20, 20); throw original; });
        Assert.Same(original, Assert.Throws<FormatException>(window.Present));
        Canvas(_ => window.Text("Reentry"));
        Assert.Throws<InvalidOperationException>(window.Present);
        Canvas(p => p.Text(0, 0, "invalid\0text", 0, 13, uint.MaxValue));
        Assert.Throws<ArgumentException>(window.Present);
        Canvas(p => p.PopClip());
        Assert.Throws<InvalidOperationException>(window.Present);
        Canvas(p => p.Circle(float.NaN, 0, 1, uint.MaxValue));
        Assert.Throws<ArgumentException>(window.Present);
        Canvas(p => p.Text(0, 0, "Recovered", 0, 13, uint.MaxValue));
        window.Present();

        void Canvas(Action<CuiCanvasPainter> paint)
        {
            Assert.True(window.NextFrame());
            window.DrawingCanvas("canvas", 1, 0, 2, 100, [], true, out _, out _, out _, out _, false, out _, out _, paint);
        }
    }

    /// <summary>Caught image validation errors leave native cache slots contiguous and reusable.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void CaughtImageErrorsDoNotAdvanceNativeCache(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Canvas image recovery", 220, 160, renderer, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);
        Draw(false, true); Draw(false, true);
        window.CapturePng(directory.File("expected.png"));
        byte[] expected = File.ReadAllBytes(directory.File("expected.png"));
        Draw(true, true); Draw(true, true);
        window.CapturePng(directory.File("recovered.png"));
        Assert.Equal(expected, File.ReadAllBytes(directory.File("recovered.png")));
        Draw(true, false);
        window.CapturePng(directory.File("removed.png"));
        Assert.NotEqual(expected, File.ReadAllBytes(directory.File("removed.png")));
        Draw(true, true);
        window.CapturePng(directory.File("restored.png"));
        Assert.Equal(expected, File.ReadAllBytes(directory.File("restored.png")));

        void Draw(bool invalid, bool visible)
        {
            Assert.True(window.NextFrame());
            window.DrawingCanvas("canvas", 1, 0, 2, 100, [], true,
                out _, out _, out _, out _, false, out _, out _, painter =>
                {
                    if (invalid)
                        for (int attempt = 0; attempt < 5; attempt++)
                            Assert.Throws<ArgumentException>(() => painter.Image(default, 0, 0, 20, 20, 0, uint.MaxValue));
                    if (visible)
                        painter.Image(new(null, 1, 1, [255, 0, 0, 255]), 20, 20, 30, 30, 0, uint.MaxValue);
                });
            window.Present();
        }
    }

    /// <summary>Retires captured state at every frame boundary.</summary>
    /// <param name="finish">Frame completion path.</param>
    [GuiTheory]
    [InlineData("present")]
    [InlineData("pump")]
    [InlineData("discard")]
    [InlineData("dispose")]
    public void PainterCapturesEndWithTheirFrame(string finish)
    {
        using var window = new CuiWindow("Painting lifetime", renderer: CuiRenderer.Software, continuous: true);
        WeakReference captured = TemporaryPaint(window, finish);
        GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect();
        Assert.False(captured.IsAlive);
    }

    private sealed class Capture { internal void Paint(CuiCanvasPainter painter) => painter.Circle(10, 10, 5, uint.MaxValue); }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static WeakReference TemporaryPaint(CuiWindow window, string finish)
    {
        var capture = new Capture();
        Assert.True(window.NextFrame());
        window.DrawingCanvas("canvas", 1, 0, 2, 100, [], true, out _, out _, out _, out _, false, out _, out _, capture.Paint);
        switch (finish)
        {
            case "present": window.Present(); break;
            case "pump": Assert.True(window.NextFrame()); break;
            case "discard": window.DiscardFrame(); break;
            default: window.Dispose(); break;
        }
        return new(capture);
    }

    /// <summary>Uses native hover/press events, suppresses idle loops and emits a leave when disabled.</summary>
    [GuiFact]
    public void PointerProposalsStopWhenIdleDisabledOrRemoved()
    {
        using var window = new CuiWindow("Canvas pointer", 320, 200, CuiRenderer.Software, continuous: true);
        var changes = new List<CuiCanvasPointer>();
        Draw(300, 180); Draw(300, 180);
        changes.Clear();
        Draw(20, 20); Assert.True(changes[^1].Hovered);
        int count = changes.Count;
        Draw(20, 20); Assert.Equal(count, changes.Count);
        Draw(20, 20, down: true); Assert.True(changes[^1].Pressed);
        Draw(20, 20, down: false);
        Draw(20, 20, enabled: false); Assert.False(changes[^1].Hovered);
        count = changes.Count;
        Draw(20, 20, enabled: false); Assert.Equal(count, changes.Count);
        Draw(20, 20);
        count = changes.Count;
        Draw(20, 20, listen: false); Assert.Equal(count, changes.Count);
        Draw(20, 20); Assert.Equal(count + 1, changes.Count);
        count = changes.Count;
        Assert.True(window.NextFrame()); window.Text("Removed"); window.Present();
        Draw(300, 180); Assert.Equal(count, changes.Count);

        void Draw(float x, float y, bool enabled = true, bool? down = null, bool listen = true)
        {
            Assert.True(window.NextFrame()); window.MovePointer(x, y);
            if (down is { } pressed) window.SetLeftButton(pressed);
            window.BeginColumn(8, 0);
            window.DrawingCanvas("canvas", 1, 0, 2, 100, [], enabled, out _, out _, out _, out _, false, out _, out _, onPointer: listen ? changes.Add : null);
            window.TrackLastControl("canvas"); window.EndColumn(); window.Present();
        }
    }
}
