using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks native tour proposals, overlay ownership and managed input validation.</summary>
public sealed class CuiTourTests
{
    /// <summary>Pointer and keyboard navigation propose changes while the caller retains the selected step.</summary>
    /// <param name="renderer">Native renderer.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void NativeNavigationPreservesTheModelAndBaseLayout(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Native guided tour", 640, 420, renderer, continuous: true);
        window.SetReducedMotion(true);
        int step = 0, focus = 0;
        float x = -1, y = -1;
        int mouse = -1, key = 0;
        Draw(); var initial = Draw();
        Assert.Equal(new[] { 0, 2 }, initial.VisibleParts);
        Assert.Equal(initial.VisibleParts, Draw(track: false).VisibleParts);
        Draw();
        Assert.Equal(120, window.GetControlRectangle("base").Height);
        var next = Click(2);
        Assert.Equal(1, next.Event); Assert.Equal(1, next.Step); Assert.Equal(0, step);
        Assert.Equal(0, Draw().Step); // The caller rejects this first proposal.
        next = Click(2); step = next.Step;
        Assert.Equal(new[] { 0, 1, 2 }, Draw().VisibleParts);
        var back = Click(1); Assert.Equal(2, back.Event); step = back.Step;
        Draw();
        key = 8; var forward = Draw(); Assert.Equal(1, forward.Event); step = forward.Step;
        Draw();
        focus = 3; Draw(); focus = 0; Draw();
        Assert.True(window.ControlFocused("part-2"));
        key = 2; forward = Draw(); Assert.Equal(1, forward.Event); step = forward.Step;
        Draw();
        var done = Click(2); Assert.Equal(3, done.Event); Assert.Equal(-1, done.Step);
        step = done.Step; Draw();
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("tour"));
        step = 0; Draw(); Draw();
        key = 3; var skipped = Draw(); Assert.Equal(4, skipped.Event); Assert.Equal(-1, skipped.Step);
        step = -1; Draw(); step = 0; Draw(); Draw();
        Assert.Equal(4, Click(0).Event);

        CuiTourResult Click(int slot)
        {
            var box = window.GetControlRectangle("part-" + slot);
            x = box.X + box.Width / 2; y = box.Y + box.Height / 2;
            mouse = 1; Draw(); mouse = 0; var result = Draw();
            x = y = -1;
            return result;
        }
        CuiTourResult Draw(bool track = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(x, y);
            if (mouse >= 0) { window.SetLeftButton(mouse == 1); mouse = -1; }
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); key = 0; }
            window.BeginGroup(false, 12, 8, 0, false);
            window.Skeleton("target", 2, 80, 80); window.TrackLastControl("target");
            window.EndGroup(); window.TrackLastControl("base");
            var result = window.Tour("tour", ["target", "target", "absent"],
                ["First##literal λ", "Second", "Centered"], ["First body", "Second body", "Last body"],
                [0, 3, 7], step, trackPart: track ? slot => "part-" + slot : null, focusPart: focus);
            window.SizeLast(false, 2, 120, 0, -1, 0, 0);
            GC.Collect(); // Native frame storage owns all borrowed UTF-8 strings until presentation.
            window.Present();
            return result;
        }
    }

    /// <summary>Malformed step arrays and native options fail before creating an overlay.</summary>
    [GuiFact]
    public void InvalidTourOptionsLeaveTheFrameUsable()
    {
        using var window = new CuiWindow("Tour validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentException>(() => window.Tour("bad", [], [], [], [], -1));
        Assert.Throws<ArgumentException>(() => window.Tour("bad", [""], [], ["Body"], [0], 0));
        Assert.Throws<ArgumentException>(() => window.Tour("bad", [""], ["zero\0"], ["Body"], [0], 0));
        Assert.Throws<ArgumentException>(() => window.Tour("bad", [""], ["Title"], ["zero\0"], [0], 0));
        foreach (int value in new[] { -2, 1 })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Tour("bad", [""], ["Title"], ["Body"], [0], value));
        foreach (int value in new[] { -1, 10 })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Tour("bad", [""], ["Title"], ["Body"], [value], 0));
        foreach (float value in new[] { -1, float.NaN, float.PositiveInfinity })
        {
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Tour("bad", [""], ["Title"], ["Body"], [0], 0, width: value));
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Tour("bad", [""], ["Title"], ["Body"], [0], 0, padding: value));
        }
        window.Tour("valid", [""], ["Title"], ["Body"], [0], 0);
        window.Present();
    }
}
