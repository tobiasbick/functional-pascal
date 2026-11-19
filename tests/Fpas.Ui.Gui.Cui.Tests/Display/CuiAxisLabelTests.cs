// [Documentation](../../../docs/ui/gui/charts/README.md)
using System.Globalization;
using System.Runtime.CompilerServices;
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Charts;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Exercises retained managed labels through native deferred painting.</summary>
public sealed class CuiAxisLabelTests
{
    /// <summary>Retains callbacks through GC, preserves UTF-8 boundaries and releases them after paint.</summary>
    /// <param name="renderer">Native rendering backend.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void LabelsSurviveGcAndMatchNativeFormatting(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Axis callbacks", 640, 360, renderer, continuous: true);
        window.SetReducedMotion(true);
        using var directory = new TemporaryDirectory();
        int axis = 0, value = 0, events = 0;
        Draw(false); Draw(false);
        window.CapturePng(directory.File("native.png"));
        Draw(true); Draw(true);
        window.CapturePng(directory.File("managed.png"));
        Assert.True(axis > 0 && value > 0 && events > 0);
        Assert.Equal(File.ReadAllBytes(directory.File("native.png")), File.ReadAllBytes(directory.File("managed.png")));

        Draw(true, new string('é', 80));
        window.CapturePng(directory.File("long.png"));
        Draw(true, new string('é', 31));
        window.CapturePng(directory.File("cut.png"));
        Assert.Equal(File.ReadAllBytes(directory.File("long.png")), File.ReadAllBytes(directory.File("cut.png")));

        void Draw(bool callbacks, string? text = null)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(310, 60);
            window.BeginColumn(8, 8);
            var dot = new CuiDotOptions(new(Height: 130, Format: new(0)), 0, 6, 2, 0, new(0), "", 0, true, true);
            var track = new CuiEventOptions(1, 0, 4, 0, 6, 0, 0, new(0), 2, true, true, true);
            if (callbacks)
            {
                dot = dot with {
                    XLabel = x => { axis++; return text ?? x.ToString("F0", CultureInfo.InvariantCulture); },
                    ValueLabel = x => { value++; return x.ToString("F0", CultureInfo.InvariantCulture); }
                };
                track = track with { Label = x => { events++; return x.ToString("F0", CultureInfo.InvariantCulture); } };
            }
            window.DotChart("chart", [1, 2, 3], [1], 3, dot, true, false);
            window.EventTrack("events", [1, 3], -1, track, true, false);
            window.EndColumn();
            GC.Collect(); GC.WaitForPendingFinalizers();
            window.Present();
        }
    }

    /// <summary>Propagates callback failures through managed presentation and permits recovery.</summary>
    [GuiFact]
    public void CallbackFailuresUnwindAfterNativePainting()
    {
        using var window = new CuiWindow("Label failure", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.DotChart("chart", [1, 2], [1], 2,
            new(new(Height: 100), 0, 6, 2, 0, null, "", 0, true, true) {
                XLabel = _ => throw new InvalidOperationException("axis failure")
            }, true, false);
        var error = Assert.Throws<InvalidOperationException>(() => window.Present());
        Assert.Equal("axis failure", error.Message);
        Assert.True(window.NextFrame());
        window.DotChart("chart", [1, 2], [1], 2,
            new(new(Height: 100), 0, 6, 2, 0, null, "", 0, true, true) { XLabel = _ => "invalid\0text" }, true, false);
        Assert.Throws<ArgumentException>(() => window.Present());
        Assert.True(window.NextFrame());
        window.DotChart("chart", [1, 2], [1], 2,
            new(new(Height: 100), 0, 6, 2, 0, null, "", 0, true, true) { XLabel = _ => window.Size.Width.ToString() }, true, false);
        Assert.Throws<InvalidOperationException>(() => window.Present());
        Assert.True(window.NextFrame());
        window.Text("Recovered"); window.Present();
    }

    /// <summary>Releases captured state after presentation, implicit pumping, discard and disposal.</summary>
    /// <param name="finish">Frame termination path.</param>
    [GuiTheory]
    [InlineData("present")]
    [InlineData("pump")]
    [InlineData("discard")]
    [InlineData("dispose")]
    public void CallbackLifetimeEndsWithItsFrame(string finish)
    {
        using var window = new CuiWindow("Label lifetime", renderer: CuiRenderer.Software, continuous: true);
        WeakReference owner = DrawTransientLabel(window, finish);
        GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect();
        Assert.False(owner.IsAlive);
    }

    private sealed class LabelOwner
    {
        internal string Format(double value) => value.ToString("F0", CultureInfo.InvariantCulture);
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static WeakReference DrawTransientLabel(CuiWindow window, string finish)
    {
        var owner = new LabelOwner();
        var weak = new WeakReference(owner);
        Assert.True(window.NextFrame());
        window.DotChart("chart", [1, 2], [1], 2,
            new(new(Height: 100), 0, 6, 2, 0, null, "", 0, true, true, owner.Format), true, false);
        switch (finish)
        {
            case "present": window.Present(); break;
            case "pump": Assert.True(window.NextFrame()); break;
            case "discard": window.DiscardFrame(); break;
            default: window.Dispose(); break;
        }
        return weak;
    }
}
