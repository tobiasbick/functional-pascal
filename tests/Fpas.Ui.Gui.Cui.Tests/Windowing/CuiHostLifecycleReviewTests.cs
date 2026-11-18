using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;
using Fpas.Ui.Gui.Cui.Widgets.Desktop;

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Checks frame completion, keyed focus and retirement of managed host state.</summary>
public sealed class CuiHostLifecycleReviewTests
{
    /// <summary>A native live-resize callback can draw while its enclosing pump retains ownership of the frame.</summary>
    [GuiFact]
    public unsafe void ResizeRendererCannotChangeTheFrameLifecycle()
    {
        using var window = new CuiWindow("Resize rendering", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame()); window.Text("Previous"); window.Present();
        Assert.True(window.NextFrame());
        bool rendered = false;
        window.SetResizeRenderer(() =>
        {
            Assert.Throws<InvalidOperationException>(() => window.NextFrame());
            Assert.Throws<InvalidOperationException>(() => window.Present());
            Assert.Throws<InvalidOperationException>(() => window.CapturePng("unused.png"));
            Assert.Throws<InvalidOperationException>(() => window.DiscardFrame());
            Assert.Throws<InvalidOperationException>(() => window.SetResizeRenderer(null));
            Assert.Throws<InvalidOperationException>(() => window.Dispose());
            window.Text("Resizing");
            rendered = true;
        });

        // Invoke the bridge callback on an active frame without a platform resize loop.
        var method = typeof(CuiWindow).GetMethod("RenderResize", BindingFlags.NonPublic | BindingFlags.Static)!;
        var render = (delegate* unmanaged[Cdecl]<nint, nint, void>)method.MethodHandle.GetFunctionPointer();
        render(0, GCHandle.ToIntPtr(GetResizeRoot(window)));
        window.DiscardFrame();
        Assert.True(window.NextFrame());
        Assert.True(rendered);
    }

    [UnsafeAccessor(UnsafeAccessorKind.Field, Name = "resizeRoot")]
    private static extern ref GCHandle GetResizeRoot(CuiWindow window);

    /// <summary>Only tracked scroll containers can be used to read native scroll metrics.</summary>
    [GuiFact]
    public void ScrollMetricsRequireATrackedScrollContainer()
    {
        using var window = new CuiWindow("Scroll metrics", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginColumn();
        window.BeginViewport("viewport", false);
        window.TrackLastControl("viewport-alias");
        window.Text("Not a viewport");
        window.TrackLastControl("text");
        window.EndScroll();
        window.BeginScroll("scroll", height: 40);
        window.TrackLastControl("scroll-alias");
        window.Gap(100);
        window.EndScroll();
        window.EndColumn();
        window.Present();

        window.ScrollMetrics("viewport-alias", out float offset, out _);
        Assert.Equal(0, offset);
        window.ScrollMetrics("scroll-alias", out _, out float maximum);
        Assert.True(maximum > 0);
        Assert.Throws<ArgumentException>("id", () => window.ScrollMetrics("text", out _, out _));
    }

    /// <summary>Capture surfaces deferred painting errors and permits a clean following frame.</summary>
    /// <param name="axis">Whether to exercise a chart label instead of a canvas painter.</param>
    [GuiTheory]
    [InlineData(false)]
    [InlineData(true)]
    public void CapturePropagatesDeferredCallbackFailures(bool axis)
    {
        using var window = new CuiWindow("Capture errors", renderer: CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        Assert.True(window.NextFrame()); window.Text("Previous"); window.Present();
        Assert.True(window.NextFrame());
        var original = new FormatException("Deferred capture failure");
        if (axis)
            window.DotChart("chart", [1, 2], [1], 2,
                new(new(Height: 100), 0, 6, 2, 0, null, "", 0, true, true) {
                    XLabel = _ => throw original
                }, true, false);
        else
            window.DrawingCanvas("canvas", 1, 0, 2, 100, [], true,
                out _, out _, out _, out _, false, out _, out _, _ => throw original);
        Assert.Same(original, Assert.Throws<FormatException>(() => window.CapturePng(directory.File("failed.png"))));
        Assert.Throws<InvalidOperationException>(() => window.Text("Closed"));
        Assert.True(window.NextFrame()); window.Text("Recovered"); window.Present();
    }

    /// <summary>Successful and failed PNG writes release captures at the frame boundary.</summary>
    /// <param name="writeFailure">Whether the destination has a missing parent directory.</param>
    [GuiTheory]
    [InlineData(false)]
    [InlineData(true)]
    public void CaptureReleasesItsFrameCallbacks(bool writeFailure)
    {
        using var window = new CuiWindow("Capture lifetime", renderer: CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        Assert.True(window.NextFrame()); window.Text("Previous"); window.Present();
        var weak = CaptureTransientPainter(window,
            directory.File(writeFailure ? "missing/capture.png" : "capture.png"), writeFailure);
        GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect();
        Assert.False(weak.IsAlive);
        Assert.True(window.NextFrame()); window.Text("Recovered"); window.Present();
    }

    private sealed class Painter
    {
        internal void Paint(CuiCanvasPainter painter) => painter.Circle(10, 10, 5, uint.MaxValue);
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static WeakReference CaptureTransientPainter(CuiWindow window, string path, bool writeFailure)
    {
        var painter = new Painter();
        Assert.True(window.NextFrame());
        window.DrawingCanvas("canvas", 1, 0, 2, 100, [], true,
            out _, out _, out _, out _, false, out _, out _, painter.Paint);
        if (writeFailure) Assert.Throws<IOException>(() => window.CapturePng(path));
        else window.CapturePng(path);
        return new(painter);
    }

    /// <summary>A public focus request resolves the same stable Unicode identity as a keyed button.</summary>
    [GuiFact]
    public void FocusRequestsResolveKeyedWidgetIdentities()
    {
        using var window = new CuiWindow("Keyed focus", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.RequestFocus("button 日本##literal");
        window.Button("button 日本##literal", "Caption", 0, 0, true, false, false, false,
            "", "", "", "", false, -1, 0);
        window.TrackLastControl("button"); window.Present();
        Assert.True(window.ControlFocused("button"));
    }

    /// <summary>A removed workspace can reapply the same model restore revision.</summary>
    /// <param name="explicitForget">Whether retirement is explicit rather than caused by omission.</param>
    [GuiTheory]
    [InlineData(false)]
    [InlineData(true)]
    public void DockRestoreRevisionEndsWithItsWorkspace(bool explicitForget)
    {
        using var window = new CuiWindow("Dock retirement", renderer: CuiRenderer.Software, continuous: true);
        const string saved = "dock1 h0.35([*left],[*right])";
        string expected = Draw();
        Assert.StartsWith("dock1 h", expected);
        if (explicitForget) window.ForgetControl("workspace");
        else
        {
            Assert.True(window.NextFrame()); window.Text("Workspace removed"); window.Present();
        }
        Assert.Equal(expected, Draw());

        string Draw()
        {
            Assert.True(window.NextFrame());
            window.RestoreDockLayout("workspace", 7, saved);
            window.BeginDockspace("workspace", new CuiDockspaceOptions(1, 0, 2, 200,
                20, 20, false, "dock1 v0.5([*right],[*left])"), true);
            foreach (string key in new[] { "left", "right" })
            {
                if (window.BeginDockPanel(key, key, "", false, true, false, null).Visible)
                { window.Text(key); window.EndDockPanel(); }
            }
            window.EndDockspace(); window.TrackLastControl("workspace");
            string layout = window.ReadDockLayout("workspace");
            Assert.NotEmpty(layout);
            window.Present();
            return layout;
        }
    }
}
