using Fpas.Ui.Gui.Cui.Tests.Packaging;

namespace Fpas.Ui.Gui.Cui.Tests.Windowing;

/// <summary>Checks native window commands, creation hints and deferred closure.</summary>
public sealed class CuiWindowCommandTests
{
    /// <summary>Consumes each native request once and leaves a deferred window open for the next decision.</summary>
    [GuiFact]
    public void DeferredNativeCloseCanBeRejectedAndRequestedAgain()
    {
        using var window = new CuiWindow("Close requests", renderer: CuiRenderer.Software, continuous: true);
        window.DeferClose = true;
        Assert.True(window.NextFrame()); window.Text("Open"); window.Present();
        for (int request = 0; request < 2; request++)
        {
            window.RequestClose();
            Assert.True(window.NextFrame());
            Assert.True(window.TakeCloseRequest());
            Assert.False(window.TakeCloseRequest());
            window.Text("Decision pending"); window.Present();
        }
        window.DeferClose = false;
        window.RequestClose();
        Assert.False(window.NextFrame());
    }

    /// <summary>Rejects invalid dimensions, Unicode and wakeup requests before calling native code.</summary>
    [Fact]
    public void InvalidCreationOptionsAreRejected()
    {
        Assert.Throws<ArgumentOutOfRangeException>(() => new CuiWindow("Min width", minimumWidth: -1));
        Assert.Throws<ArgumentOutOfRangeException>(() => new CuiWindow("Min height", minimumHeight: -1));
        Assert.Throws<ArgumentException>(() => new CuiWindow("App ID", appId: "bad\0id"));
    }

    /// <summary>Applies commands on the owner and closes their lifetime with the window.</summary>
    [GuiFact]
    public void WindowCommandsEnforceOwnerAndLifetime()
    {
        using var window = new CuiWindow("Owner commands", renderer: CuiRenderer.Software, continuous: true);
        Assert.Throws<ArgumentException>(() => window.SetTitle("bad\0title"));
        foreach (float delay in new[] {-1f, float.NaN, float.PositiveInfinity})
            Assert.Throws<ArgumentOutOfRangeException>(() => window.RequestFrame(delay));
        Task.Factory.StartNew(() =>
        {
            Assert.Throws<InvalidOperationException>(() => window.SetTitle("Worker"));
            Assert.Throws<InvalidOperationException>(() => window.RequestFrame());
            Assert.Throws<InvalidOperationException>(() => window.TakeCloseRequest());
        }, CancellationToken.None, TaskCreationOptions.LongRunning, TaskScheduler.Default).GetAwaiter().GetResult();
        window.RequestFrame(); window.RequestFrame(.02f);
        Assert.True(window.NextFrame()); window.Text("Commands"); window.Present();
        window.Dispose();
        Assert.Throws<ObjectDisposedException>(() => window.SetTitle("Closed"));
        Assert.Throws<ObjectDisposedException>(() => window.RequestFrame());
        Assert.Throws<ObjectDisposedException>(() => window.TakeCloseRequest());
    }

    /// <summary>Checks real X11 identity/title properties and min/max hints supplied by CUI.</summary>
    [X11GuiFact]
    public void X11ReceivesIdentityTitleAndCreationHints()
    {
        const string title = "Native window properties";
        using (var window = new CuiWindow(title, 420, 300, CuiRenderer.Software, true,
            minimumWidth: 240, minimumHeight: 180, appId: "org.example.FpasWindow"))
        {
            using var input = new X11Input(title);
            Assert.Contains("org.example.FpasWindow", input.TextProperty("WM_CLASS"));
            long[] hints = input.IntegerProperty("WM_NORMAL_HINTS");
            Assert.Equal(240, hints[5]); Assert.Equal(180, hints[6]);
            window.SetTitle("Changed — fenêtre");
            for (int frame = 0; frame < 3; frame++)
            {
                Assert.True(window.NextFrame()); window.Text("Native title"); window.Present();
            }
            Assert.Equal("Changed — fenêtre", input.TextProperty("_NET_WM_NAME"));
            window.Resize(460, 320);
            for (int frame = 0; frame < 3; frame++)
            {
                Assert.True(window.NextFrame()); window.Text("Native size"); window.Present();
            }
            Assert.Equal(460, window.Size.Width); Assert.Equal(320, window.Size.Height);
        }
        using (var window = new CuiWindow(title, 420, 300, CuiRenderer.Software, true, fixedSize: true))
        {
            using var input = new X11Input(title);
            long[] hints = input.IntegerProperty("WM_NORMAL_HINTS");
            Assert.Equal(420, hints[5]); Assert.Equal(300, hints[6]);
            Assert.Equal(hints[5], hints[7]); Assert.Equal(hints[6], hints[8]);
        }
    }
}
