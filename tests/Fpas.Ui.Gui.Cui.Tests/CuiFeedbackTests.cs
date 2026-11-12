namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises native CUI feedback, lifetime and literal labels through both renderers.</summary>
public sealed class CuiFeedbackTests
{
    /// <summary>Dismissal is native, stable across captions, delivered once, and retired on removal.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, 2)]
    [InlineData(CuiRenderer.OpenGl, 2)]
    [InlineData(CuiRenderer.Software, 6)]
    [InlineData(CuiRenderer.OpenGl, 6)]
    [InlineData(CuiRenderer.Software, 0)]
    [InlineData(CuiRenderer.OpenGl, 0)]
    public void AlertsRetainOnlyTheirNativeDismissal(CuiRenderer renderer, int activation)
    {
        using var window = new CuiWindow("Native feedback", 600, 480, renderer, continuous: true);
        window.SetReducedMotion(true);
        Assert.Equal(1, Draw(focus: true));
        Assert.True(window.ControlFocused("dismiss"));
        Assert.True(window.GetControlRectangle("alert").Height > 50);
        Assert.Equal(1, Draw(title: "Renamed ### literally"));
        Assert.True(window.ControlFocused("dismiss"));
        var button = window.GetControlRectangle("dismiss");
        Assert.Equal(1, Draw(enabled: false, key: activation));
        Assert.False(window.ControlFocused("dismiss"));
        Draw(focus: true);
        if (activation == 0)
        {
            Draw(pointer: button);
            Draw(pointer: button, mouse: true);
            Assert.Equal(2, Draw(pointer: button, mouse: false));
        }
        else Assert.Equal(2, Draw(key: activation));
        Assert.Equal(0, Draw(title: "Renamed ### still hidden"));
        Assert.Equal(0, Draw(focus: true));
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("dismiss"));
        Assert.Equal(0, window.GetControlRectangle("alert").Height);
        Draw(remove: true);
        Assert.Equal(1, Draw());
        Assert.True(window.GetControlRectangle("alert").Height > 50);

        int Draw(bool focus = false, bool enabled = true, int key = 0, bool remove = false,
            string title = "Heading ## literal – λ", CuiRectangle? pointer = null, bool? mouse = null)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(pointer is { } p ? p.X + 12 : -1, pointer is { } q ? q.Y + 12 : -1);
            if (mouse is { } down) window.SetLeftButton(down);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn(12, 10);
            window.Progress("before", .68f, label: "Uploading ## literally", showValue: true);
            window.TrackLastControl("before");
            int result = remove ? 0 : window.Alert("alert", title, "Native supporting text", "", 3,
                true, enabled, focus, "dismiss");
            if (!remove)
            {
                Assert.Equal(result != 0, window.HasLastWidget);
                // Hidden alerts must not redirect sizing or styling to their preceding sibling.
                window.SizeLast(false, 2, 80, 0, -1, 0, 0);
                window.StyleLast(1);
            }
            window.Spinner("spinner", 22);
            window.TrackLastControl("spinner");
            window.EndColumn();
            window.Present();
            Assert.Equal(22, window.GetControlRectangle("spinner").Width);
            Assert.True(window.GetControlRectangle("before").Height < 80);
            return result;
        }
    }

    /// <summary>Native dimensions and optional caption rows survive layout and range updates.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ProgressAndSpinnerKeepNativeGeometry(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Progress geometry", 500, 500, renderer, continuous: true);
        window.SetReducedMotion(true);
        foreach (int width in new[] { 500, 280, 700, 280 })
        {
            window.Resize(width, 500);
            Draw(); Draw();
            Assert.Equal(width - 24, window.GetControlRectangle("bar").Width);
            Assert.Equal(12, window.GetControlRectangle("bar").Height);
            Assert.Equal(28, window.GetControlRectangle("spinner").Height);
            Assert.True(window.GetControlRectangle("activity").Height > 6);
        }

        void Draw()
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(12, 10);
            window.Progress("bar", 1, 2, height: 12);
            window.TrackLastControl("bar");
            window.Progress("activity", 0, 1, indeterminate: true, label: "Index ## λ", showValue: true);
            window.TrackLastControl("activity");
            window.Spinner("spinner", 28);
            window.TrackLastControl("spinner");
            window.EndColumn();
            window.Present();
        }
    }

    /// <summary>Invalid public bridge options fail before native state or nodes are changed.</summary>
    [GuiFact]
    public void InvalidOptionsLeaveTheFrameUsable()
    {
        using var window = new CuiWindow("Feedback validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        foreach (float value in new[] { float.NaN, float.PositiveInfinity, -.1f, 1.1f })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.Progress("bad", value));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Progress("bad", 0, tone: 9));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Progress("bad", 0, height: float.NaN));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.Spinner("bad", float.PositiveInfinity));
        Assert.Throws<ArgumentException>(() => window.Progress("bad", 0, label: "zero\0"));
        Assert.Throws<ArgumentException>(() => window.Alert("bad", " ", "", "", 0, false, true, false, "close"));
        window.Progress("valid", .5f);
        window.Spinner("valid-spinner");
        window.Present();
    }
}
