namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks model-owned status, native activation and literal caption geometry.</summary>
public sealed class CuiStatusButtonTests
{
    /// <summary>Suppresses non-idle activation while preserving focus through state and caption changes.</summary>
    /// <param name="size">Native size ordinal.</param>
    /// <param name="height">Expected logical control height.</param>
    [GuiTheory]
    [InlineData(0, 36)]
    [InlineData(1, 30)]
    [InlineData(2, 44)]
    public void StatusIsControlledAndFocusSurvivesAllStates(int size, int height)
    {
        using var window = new CuiWindow("Status button", 640, 480, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.False(Draw(0, focus: true));
        Assert.Equal(height, window.GetControlRectangle("status").Height);
        float width = window.GetControlRectangle("status").Width;
        Assert.True(width > 100, "The literal caption must determine the width.");
        Assert.True(Draw(0, key: 2));
        Assert.False(Draw(1, key: 6));
        Assert.True(window.ControlFocused("status"));
        Assert.False(Draw(2, key: 2));
        Assert.True(window.ControlFocused("status"));
        Assert.False(Draw(3, key: 6));
        Assert.True(window.ControlFocused("status"));
        Assert.False(Draw(3));
        Assert.True(Draw(0, key: 6, label: "Renamed ### literally"));
        Assert.True(window.ControlFocused("status"));
        Assert.False(Draw(0, enabled: false, key: 2));
        Assert.False(window.ControlFocused("status"));
        Assert.True(Draw(0, focus: true, key: 2));
        window.ForgetControl("status");
        window.SetReducedMotion(false);
        Assert.False(Draw(0, focus: true));
        Assert.Equal(width, window.GetControlRectangle("status").Width);

        bool Draw(int state, int key = 0, bool enabled = true, bool focus = false,
            string label = "Save ## literally")
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            bool clicked = window.StatusButton("status", label, state, "Saving ### literally", "Saved ## literally",
                "Failed ## literally", "", 1, size, enabled, focus);
            window.TrackLastControl("status");
            window.EndColumn();
            window.Present();
            return clicked;
        }
    }

    /// <summary>Rejects invalid native states, styles, sizes and UTF-8 input before drawing.</summary>
    [GuiFact]
    public void StatusValidatesStateAppearanceAndCaptions()
    {
        using var window = new CuiWindow("Status validation", 480, 320, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(state: -1));
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(state: 4));
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(variant: 3));
        Assert.Throws<ArgumentOutOfRangeException>(() => Draw(size: 3));
        Assert.Throws<ArgumentException>(() => Draw(busy: "Bad\0label"));
        Assert.Throws<ArgumentException>(() => Draw(success: "Bad\0label"));
        Assert.Throws<ArgumentException>(() => Draw(error: "Bad\0label"));

        void Draw(int state = 0, int variant = 1, int size = 0,
            string busy = "Busy", string success = "Done", string error = "Failed") =>
            window.StatusButton("id", "Save", state, busy, success, error, "", variant, size, true, false);
    }
}
