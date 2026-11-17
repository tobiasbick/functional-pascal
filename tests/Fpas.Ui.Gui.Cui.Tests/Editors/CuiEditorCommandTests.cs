using Fpas.Ui.Gui.Cui.Widgets.Editors;

namespace Fpas.Ui.Gui.Cui.Tests.Editors;

/// <summary>Exercises native editor commands with UTF-16 callers and deferred native highlights.</summary>
public sealed class CuiEditorCommandTests
{
    /// <summary>Selections, find and goto preserve Unicode offsets and work in read-only editors.</summary>
    /// <param name="renderer">Native renderer used for deferred paint.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void NativeCommandsReportScalarBoundariesAndFindState(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Editor commands", 600, 420, renderer, continuous: true);
        window.SetReducedMotion(true);
        const string text = "a😀é\tz\nsecond 😀\nthird";
        Draw();
        window.CodeEditorSelect("editor", 1, 4);
        var selected = Assert.IsType<CuiCodeEditorEvent>(Draw());
        Assert.Equal(1, selected.Anchor); Assert.Equal(4, selected.Cursor); Assert.Null(selected.Text);
        window.CodeEditorSelect("editor", 4, 1);
        selected = Assert.IsType<CuiCodeEditorEvent>(Draw());
        Assert.Equal(4, selected.Anchor); Assert.Equal(1, selected.Cursor);
        window.CodeEditorGoTo("editor", 2, 2);
        var moved = Assert.IsType<CuiCodeEditorEvent>(Draw());
        Assert.Equal(2, moved.Line); Assert.Equal(2, moved.Column); Assert.Equal(8, moved.Cursor);
        window.CodeEditorFind("editor", "😀");
        var found = Assert.IsType<CuiCodeEditorEvent>(Draw());
        Assert.Equal(2, found.Matches); Assert.Equal(2, found.MatchIndex); Assert.True(found.Focused);
        window.CodeEditorFind("editor", null);
        Assert.True(Assert.IsType<CuiCodeEditorEvent>(Draw()).Focused);
        // Mixed requests have one canonical outcome: the last request wins.
        window.CodeEditorGoTo("editor", 3);
        window.CodeEditorSelect("editor", 0, text.Length);
        selected = Assert.IsType<CuiCodeEditorEvent>(Draw());
        Assert.Equal(0, selected.Anchor); Assert.Equal(text.Length, selected.Cursor);
        window.CodeEditorGoTo("editor", int.MaxValue, int.MaxValue);
        moved = Assert.IsType<CuiCodeEditorEvent>(Draw());
        Assert.Equal(3, moved.Line); Assert.Equal(text.Length, moved.Cursor);

        CuiCodeEditorEvent? Draw()
        {
            Assert.True(window.NextFrame());
            var result = window.CodeEditor("editor", text, 1024, "text", "", 1, 0, 340, 0, 4, 3,
                [new(1, 4), new(7, 13, 2)]);
            window.TrackLastControl("editor");
            GC.Collect();
            window.Present();
            return result;
        }
    }

    /// <summary>Requests for removed/disabled controls expire and invalid Unicode boundaries fail before native selection.</summary>
    [GuiFact]
    public void RequestsHaveOneViewLifetimeAndRejectSplitSurrogates()
    {
        using var window = new CuiWindow("Editor request lifetime", renderer: CuiRenderer.Software, continuous: true);
        window.CodeEditorGoTo("absent", 9);
        Assert.True(window.NextFrame()); window.Text("No editor"); window.Present();
        Assert.True(window.NextFrame());
        var result = Assert.IsType<CuiCodeEditorEvent>(window.CodeEditor("absent", "a😀\nb", 128, "text", "", 1, 0, 200, 0, 4, 1));
        Assert.Equal(1, result.Line); window.Present();
        window.CodeEditorGoTo("absent", 2);
        Assert.True(window.NextFrame());
        window.CodeEditor("absent", "a😀\nb", 128, "text", "", 1, 0, 200, 0, 4, 0);
        window.Present();
        Assert.True(window.NextFrame());
        result = window.CodeEditor("absent", "a😀\nb", 128, "text", "", 1, 0, 200, 0, 4, 1);
        Assert.True(result is null || result.Line == 1); window.Present();
        window.CodeEditorSelect("absent", 0, 2);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentOutOfRangeException>(() => window.CodeEditor("absent", "a😀", 128, "text", "", 1, 0, 200, 0, 4, 1));
        window.DiscardFrame();
        window.CodeEditorSelect("absent", 0, 10);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentOutOfRangeException>(() => window.CodeEditor("absent", "short", 128, "text", "", 1, 0, 200, 0, 4, 1));
        window.DiscardFrame();
        Assert.Throws<ArgumentException>(() => window.CodeEditorFind("absent", new string('é', 128)));
        Assert.Throws<ArgumentException>(() => window.CodeEditorFind("absent", "bad\0query"));
        Task.Factory.StartNew(() => Assert.Throws<InvalidOperationException>(() => window.CodeEditorGoTo("absent", 1)),
            CancellationToken.None, TaskCreationOptions.LongRunning, TaskScheduler.Default).GetAwaiter().GetResult();
        window.Dispose();
        Assert.Throws<ObjectDisposedException>(() => window.CodeEditorFind("absent", "query"));
    }

    /// <summary>Rejects invalid marks, while disabling input does not manufacture repeated focus observations.</summary>
    [GuiFact]
    public void InvalidMarksAndDisabledFramesPreserveObservationState()
    {
        using var window = new CuiWindow("Editor marks", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        foreach (CuiTextMark[] marks in new[] { new[] { new CuiTextMark(2, 3) },
            [new(0, 1, 7)], [new(3, 4), new(0, 1)] })
            Assert.ThrowsAny<ArgumentException>(() => window.CodeEditor("editor", "a😀b", 128, "text", "", 1, 0, 200, 0, 4, 1, marks));
        Assert.IsType<CuiCodeEditorEvent>(window.CodeEditor("editor", "source", 128, "text", "", 1, 0, 200, 0, 4, 17));
        window.Present();
        Assert.True(window.NextFrame());
        window.CodeEditor("editor", "source", 128, "text", "", 1, 0, 200, 0, 4, 0);
        window.Present();
        Assert.True(window.NextFrame());
        Assert.False(Assert.IsType<CuiCodeEditorEvent>(window.CodeEditor("editor", "source", 128, "text", "", 1, 0, 200, 0, 4, 1)).Focused);
        window.Present();
        Assert.True(window.NextFrame());
        Assert.Null(window.CodeEditor("editor", "source", 128, "text", "", 1, 0, 200, 0, 4, 1));
        window.Present();
    }
}
