using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises native edit transitions across the four Smart inputs bindings.</summary>
public sealed class CuiSmartInputTests
{
    /// <summary>Matches native drag increments, Shift precision and click-to-type behavior.</summary>
    /// <param name="renderer">The native renderer under test.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ScrubUsesNativePointerDraggingAndFineSteps(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Scrub pointer", 520, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        float value = 50;
        Draw(-1, -1);
        var bounds = window.GetControlRectangle("scrub");
        float x = bounds.X + 40, y = bounds.Y + bounds.Height / 2;
        Draw(x, y, down: true);
        Draw(x + 10, y);
        Assert.Equal(55, value);
        Draw(x + 10, y, down: false);
        Draw(x + 10, y, down: true, modifiers: 1);
        Draw(x + 20, y, modifiers: 1);
        Assert.Equal(55.5f, value);
        Draw(x + 20, y, down: false, modifiers: 1);
        Draw(x + 20, y, down: true);
        Draw(x + 20, y, down: false);
        Draw(x + 20, y, text: "17.25");
        Draw(x + 20, y, key: 2);
        Assert.Equal(17.25f, value);

        void Draw(float mx, float my, bool? down = null, int modifiers = 0, string? text = null, int key = 0)
        {
            Assert.True(window.NextFrame()); window.MovePointer(mx, my); window.SetKey(0, false, modifiers);
            if (down is { } pressed) window.SetLeftButton(pressed);
            Press(window, key);
            if (text is not null) window.InputText(text);
            window.BeginColumn();
            value = window.ScrubField("scrub", "Opacity", value, 0, 100, .5f, 2, "%", "", 0, 0, true, false, true);
            window.EndColumn(); GC.Collect(); window.Present();
        }
    }

    /// <summary>Retains uncommitted Unicode drafts, commits or cancels them, and retires native state.</summary>
    /// <param name="renderer">The native renderer under test.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void InlineDraftsRemainNativeUntilCommit(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Inline edit", 520, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        string model = "Roadmap";
        Draw(focus: true);
        Assert.True(Draw(key: 2).Editing);
        var draft = Draw(text: "Plan 日本語");
        Assert.Equal("Roadmap", draft.Value);
        Assert.Equal("Plan 日本語", draft.Draft);
        Assert.Equal(draft.Draft, Draw().Draft);
        Assert.True(window.ControlFocused("title"));
        Assert.Equal("Plan 日本語", Draw(key: 2).Value);
        model = "Plan 日本語";
        Draw(key: 2);
        Draw(text: "Discard");
        Assert.Equal(model, Draw(key: 3).Draft);
        Draw();
        Draw(key: 2);
        Draw(text: "Old draft");
        model = "External replacement";
        Assert.Equal(model, Draw().Draft);
        Draw(key: 3);
        Draw();
        Assert.Equal(model, Draw(key: 2).Value);
        Draw(text: "Rejected");
        Assert.Equal("Rejected", Draw(key: 2).Value);
        Assert.Equal(model, Draw().Draft);
        window.ForgetControl("title");
        Assert.False(Draw().Editing);

        CuiInlineEditResult Draw(bool focus = false, int key = 0, string? text = null)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            Press(window, key);
            if (text is not null) window.InputText(text);
            window.BeginColumn();
            var result = window.InlineEdit("title", model, "Add an owner", 2, 26, 127, 0, 0, true, focus, true);
            window.EndColumn();
            GC.Collect();
            window.Present();
            return result;
        }
    }

    /// <summary>Uses native keys, numeric typing, cancellation, clamping and disabled admission.</summary>
    /// <param name="renderer">The native renderer under test.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ScrubSwitchesBetweenDragKeysAndTyping(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Scrub", 520, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        float model = 40;
        Draw(focus: true);
        model = Draw(key: 8);
        Assert.Equal(41, model);
        Assert.Equal(model, Draw(key: 8, enabled: false));
        Draw(focus: true);
        Draw(key: 2);
        Draw(text: "17.5");
        Assert.Equal(17.5f, Draw(key: 2));
        Assert.True(window.ControlFocused("scrub"));
        model = 17.5f;
        Draw(); Draw(key: 2); Draw(text: "99");
        Assert.Equal(model, Draw(key: 3));
        Draw(); Draw(key: 2); Draw(text: "9999");
        Assert.Equal(100, Draw(key: 2));
        Draw(); Draw(key: 2); Draw(text: "nan");
        Assert.Equal(model, Draw(key: 2));
        window.ForgetControl("scrub");
        Assert.Equal(model, Draw());

        float Draw(bool focus = false, int key = 0, string? text = null, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1); Press(window, key);
            if (text is not null) window.InputText(text);
            window.BeginColumn();
            float result = window.ScrubField("scrub", "Radius ## literal", model, 0, 100, 1, 1,
                "px", "", 2, 180, enabled, focus, true);
            window.EndColumn(); GC.Collect(); window.Present();
            return result;
        }
    }

    /// <summary>Reconciles external and rejected amounts while focused and preserves decimal precision.</summary>
    /// <param name="renderer">The native renderer under test.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void MoneyUsesNativeDigitsAndControlledValues(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Money", 520, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        double model = 1249.5;
        Draw(focus: true);
        Assert.Equal(42.75, Replace("42.75"));
        model = 42.75;
        Draw();
        model = 123.5;
        Assert.Equal(123.5, Draw());
        Assert.Equal(7.25, Replace("7.25"));
        Assert.Equal(model, Draw());
        Assert.Equal(model, Replace("99", enabled: false));
        Draw(focus: true);
        Assert.Equal(model, Replace(new string('9', 63)));

        double Replace(string text, bool enabled = true)
        {
            Draw(select: true, enabled: enabled);
            return Draw(text: text, enabled: enabled);
        }

        double Draw(bool focus = false, string? text = null, bool select = false, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (select) Press(window, 15, 2);
            if (text is not null) window.SetKey(0, false, 0);
            if (text is not null) window.InputText(text);
            window.BeginColumn();
            double result = window.MoneyField("amount", model, "Amount", "$", "USD", "", 2, 2, 240,
                enabled, focus, true);
            window.EndColumn(); GC.Collect(); window.Present();
            return result;
        }
    }

    /// <summary>Records, cancels and clears combinations using native key names and modifier ordering.</summary>
    /// <param name="renderer">The native renderer under test.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void RecorderCapturesKeysAndOwnsItsClearAction(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Shortcut recorder", 520, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        CuiKeyCombination model = new(30, 3);
        Draw(focus: true);
        Draw(key: 2);
        var recorded = Draw(key: 20, mods: 6);
        Assert.Equal(new CuiKeyCombination(20, 6), recorded.Value);
        model = recorded.Value;
        Assert.True(Draw().ClearVisible);
        Draw(key: 6); Draw(key: 3);
        Assert.Equal(model, Draw().Value);
        Draw(key: 2);
        Assert.Equal(new CuiKeyCombination(0, 0), Draw(key: 4).Value);
        model = new(30, 3);
        Assert.True(Draw(focusClear: true).ClearVisible);
        Assert.True(window.ControlFocused("clear"));
        Assert.Equal(new CuiKeyCombination(0, 0), Draw(key: 2).Value);
        model = new(0, 0);
        Draw(focus: true); Draw(key: 2); Draw(key: 3);
        Assert.Equal(model, Draw().Value);
        window.ForgetControl("shortcut");
        Assert.Equal(model, Draw().Value);

        CuiShortcutRecorderResult Draw(bool focus = false, int key = 0, int mods = 0, bool focusClear = false)
        {
            Assert.True(window.NextFrame()); window.MovePointer(-1, -1); Press(window, key, mods);
            window.BeginColumn();
            var result = window.ShortcutRecorder("shortcut", model, "Record shortcut", 0, 0, true, focus, "clear", focusClear);
            string label = window.ShortcutLabel(new(30, 3));
            Assert.Equal("Ctrl Shift P", label);
            Assert.Equal("Ctrl Shift P", window.DrawShortcutLabel(new(30, 3)));
            window.EndColumn(); GC.Collect(); window.Present();
            return result;
        }
    }

    /// <summary>Rejects malformed options before changing native frame state.</summary>
    [GuiFact]
    public void InvalidSmartOptionsLeaveTheFrameUsable()
    {
        using var window = new CuiWindow("Invalid smart inputs", 520, 320, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentOutOfRangeException>(() => window.InlineEdit("id", "text", "", 1, 0, 512, 0, 0, true, false, true));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ScrubField("id", "Radius", 1, 0, 100, float.NaN, 0, "", "", 0, 0, true, false, true));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.MoneyField("id", 1e100, "", "$", "", "", 2, 0, 0, true, false, true));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ShortcutRecorder("id", new(68, 0), "", 0, 0, true, false, "clear"));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ShortcutLabel(new(0, 2)));
        window.Button("Frame remains usable"); window.Present();
    }

    private static void Press(CuiWindow window, int key, int mods = 0)
    {
        if (key == 0) return;
        window.SetKey(key, true, mods); window.SetKey(key, false, mods);
    }
}
