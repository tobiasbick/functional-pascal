// [Documentation](../../../docs/ui/gui/text-and-display/canvas.md)
using Fpas.Ui.Gui.Cui.Tests.Packaging;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks native focus ownership and ordered delivery across an authored pane.</summary>
public sealed class CuiCanvasInputTests
{
    /// <summary>Preserves repeat and Unicode ordering across Tab and drops events for removed canvases.</summary>
    [GuiFact]
    public void RoutesOrderedInputAcrossScopedCanvases()
    {
        using var window = new CuiWindow("Canvas input", 400, 240, CuiRenderer.Software, continuous: true);
        var events = new List<(string Id, CuiCanvasInput Value)>();
        bool focused = false, visible = false;
        Frame(request: true);
        Frame();
        events.Clear();

        Frame(() =>
        {
            window.SetKey(15, true, 1);
            window.SetKey(15, true, 1);
            window.SetKey(15, false, 1);
            window.InputText("ä🎵");
            window.SetKey(1, true);
            window.SetKey(1, false);
            window.InputText("次");
        });

        Assert.Equal(new[] { "first", "first", "first", "first", "second" }, events.Select(e => e.Id));
        Assert.Equal(new CuiCanvasInput(CuiCanvasInputKind.Key, 15, 1, true, false, ""), events[0].Value);
        Assert.True(events[1].Value.Repeated);
        Assert.False(events[2].Value.Down);
        Assert.Equal("ä🎵", events[3].Value.Text);
        Assert.Equal("次", events[4].Value.Text);
        Assert.True(focused);
        Assert.True(visible);
        events.Clear();
        Frame(() =>
        {
            window.SetKey(1, true, 1);
            window.SetKey(1, false, 1);
            window.InputText("back");
        });
        Assert.Equal(("first", "back"), Assert.Single(events.Select(e => (e.Id, e.Value.Text))));
        events.Clear();
        Frame(() => { window.SetKey(1, true); window.SetKey(1, false); });
        Frame(() => window.InputText("removed"), includeSecond: false);
        Assert.Empty(events);

        void Frame(Action? inject = null, bool request = false, bool includeSecond = true)
        {
            Assert.True(window.NextFrame());
            inject?.Invoke();
            window.BeginColumn(0, 0);
            window.PushFocusScope("pane", true);
            Canvas("first", request);
            window.PushFocusScope("inactive", false);
            Canvas("outside", false);
            window.PopFocusScope();
            if (includeSecond) Canvas("second", false);
            window.PopFocusScope();
            window.EndColumn();
            window.Present();
        }

        void Canvas(string id, bool request)
        {
            window.DrawingCanvas(id, 1, 0, 2, 40, [], true, out _, out _, out _, out _, false, out _, out _,
                paint: p => { if (id == "second") (focused, visible) = (p.Focused, p.FocusVisible); },
                focus: request, cursor: 1, onInput: value => events.Add((id, value)));
        }
    }

    /// <summary>Opt-in activation handles Enter, while discarded component input has no effect.</summary>
    [GuiFact]
    public void ActivatesFocusableCanvasAndDropsDiscardedInput()
    {
        using var window = new CuiWindow("Keyboard activation", 320, 240, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        Draw(focus: true);
        window.Present();
        Assert.True(window.NextFrame());
        window.SetKey(2, true);
        Assert.True(Draw());
        window.SetKey(2, false);
        window.Present();

        var events = new List<CuiCanvasInput>();
        Assert.True(window.NextFrame());
        Draw(focus: true, onInput: events.Add);
        window.Present();
        Assert.True(window.NextFrame());
        window.InputText("discard");
        Draw(onInput: events.Add);
        window.DiscardFrame();
        Assert.True(window.NextFrame());
        Draw(onInput: events.Add);
        window.Present();
        Assert.Empty(events);

        bool Draw(bool focus = false, Action<CuiCanvasInput>? onInput = null) =>
            window.DrawingCanvas("draft", 1, 0, 2, 40, [], true, out _, out _, out _, out _, false, out _, out _,
                focusable: true, focus: focus, onInput: onInput);
    }

    /// <summary>An input callback failure closes the frame at every presentation entry point.</summary>
    /// <param name="ending">The operation that completes the submitted view.</param>
    [GuiTheory]
    [InlineData("Present")]
    [InlineData("NextFrame")]
    [InlineData("CapturePng")]
    public void InputFailureDiscardsFrame(string ending)
    {
        using var window = new CuiWindow("Failed canvas input", 320, 240, CuiRenderer.Software, continuous: true);
        using var directory = new Fpas.Tests.Support.TemporaryDirectory();
        var error = new FormatException("callback");
        var events = new List<string>();
        Assert.True(window.NextFrame());
        Draw(true, _ => { });
        window.Present();
        Assert.True(window.NextFrame());
        window.InputText("fail");
        window.InputText("queued");
        Draw(false, value => { events.Add(value.Text); throw error; });
        Assert.Same(error, Assert.Throws<FormatException>(() =>
        {
            switch (ending)
            {
                case "Present": window.Present(); break;
                case "NextFrame": window.NextFrame(); break;
                case "CapturePng": window.CapturePng(directory.File("frame.png")); break;
            }
        }));
        Assert.Throws<InvalidOperationException>(() => window.Text("closed"));
        Assert.True(window.NextFrame());
        Draw(true, value => events.Add(value.Text));
        window.Present();
        Assert.True(window.NextFrame());
        window.InputText("recovered");
        Draw(false, value => events.Add(value.Text));
        window.Present();
        Assert.Equal(new[] { "fail", "recovered" }, events);

        void Draw(bool focus, Action<CuiCanvasInput> callback) =>
            window.DrawingCanvas("canvas", 1, 0, 2, 40, [], true, out _, out _, out _, out _, false, out _, out _,
                focus: focus, onInput: callback);
    }

    /// <summary>Window focus gates text, clears key repeat, and respects the current enabled state.</summary>
    [X11GuiFact]
    public void RoutesWindowFocusChangesAndDropsDisabledInput()
    {
        using var window = new CuiWindow("Canvas focus owner", 320, 240, CuiRenderer.Software, continuous: true);
        using var other = new CuiWindow("Canvas focus target", 320, 240, CuiRenderer.Software, continuous: true);
        using var target = new X11Input("Canvas focus target");
        using var owner = new X11Input("Canvas focus owner");
        var events = new List<CuiCanvasInput>();
        Frame(focus: true);
        Frame();
        Frame(() => window.SetKey(15, true));
        events.Clear();
        target.Focus();
        Frame(() => window.InputText("blurred"));
        owner.Focus();
        Frame(() => { window.SetKey(15, true); window.SetKey(15, false); });
        Assert.Collection(events,
            lost => Assert.Equal((CuiCanvasInputKind.WindowFocus, false), (lost.Kind, lost.Down)),
            gained => Assert.Equal((CuiCanvasInputKind.WindowFocus, true), (gained.Kind, gained.Down)),
            down => Assert.Equal((CuiCanvasInputKind.Key, 15, true, false), (down.Kind, down.Key, down.Down, down.Repeated)),
            up => Assert.Equal((CuiCanvasInputKind.Key, 15, false), (up.Kind, up.Key, up.Down)));
        events.Clear();
        Frame(() => window.InputText("disabled"), enabled: false);
        Assert.Empty(events);

        void Frame(Action? inject = null, bool focus = false, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            inject?.Invoke();
            window.DrawingCanvas("canvas", 1, 0, 2, 40, [], enabled, out _, out _, out _, out _, false, out _, out _,
                focus: focus, onInput: events.Add);
            window.Present();
        }
    }

    /// <summary>Routes key, text and key-up from the actual X11 platform input hook.</summary>
    [X11GuiFact]
    public void PumpsPlatformInputIntoFocusedCanvas()
    {
        const string title = "Canvas platform input";
        using var window = new CuiWindow(title, 400, 240, CuiRenderer.Software, continuous: true);
        var events = new List<CuiCanvasInput>();
        Frame(true);
        using var input = new X11Input(title);
        Frame(false);
        events.Clear();
        input.Key("a", true);
        input.Key("a", false);
        for (int frame = 0; frame < 60 && !events.Any(e => e.Kind == CuiCanvasInputKind.Key && !e.Down); frame++)
            Frame(false);
        Assert.Collection(events.Where(e => e.Kind != CuiCanvasInputKind.WindowFocus),
            down => Assert.True(down.Kind == CuiCanvasInputKind.Key && down.Down && !down.Repeated),
            text => Assert.Equal("a", text.Text),
            up => Assert.True(up.Kind == CuiCanvasInputKind.Key && !up.Down));

        void Frame(bool request)
        {
            Assert.True(window.NextFrame());
            window.DrawingCanvas("input", 1, 0, 2, 40, [], true, out _, out _, out _, out _, false, out _, out _,
                focus: request, onInput: events.Add);
            window.Present();
        }
    }
}
