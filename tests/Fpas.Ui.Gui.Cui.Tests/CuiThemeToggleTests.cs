namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks stable controlled light/dark actions and their native input paths.</summary>
public sealed class CuiThemeToggleTests
{
    /// <summary>Retains focus across glyph changes, defers palette changes and cancels disabled presses.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ModelOwnsAppearanceAndIdentity(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Theme toggle", 480, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        uint[] palette = window.ReadTheme();
        palette[0] = 0xff352719;
        palette[4] = 0xffc7d9eb;
        window.SetTheme(true, 2, palette);
        bool secondActivated = false;
        Assert.False(Draw(dark: false, focus: true));
        Assert.True(window.ControlFocused("first"));
        var rectangle = window.GetControlRectangle("first");
        Assert.Equal(rectangle.Height, rectangle.Width);
        Assert.True(Draw(dark: false, key: 2));
        Assert.False(Draw(dark: true));
        Assert.True(window.ControlFocused("first"));
        Assert.Equal(rectangle, window.GetControlRectangle("first"));
        Assert.True(Draw(dark: true, key: 6));
        Assert.True(Draw(dark: true, key: 2)); // Rejecting a proposal keeps the model's state.
        Assert.Equal(palette, window.ReadTheme());
        Assert.False(Draw(dark: true, enabled: false, key: 2, focus: true));
        Assert.False(window.ControlFocused("first"));
        Assert.False(Draw(dark: false, focus: true));

        var second = window.GetControlRectangle("second");
        Assert.False(Draw(dark: false, point: second, press: true));
        Assert.False(Draw(dark: false, point: second, press: false));
        Assert.True(secondActivated);
        Assert.True(window.ControlFocused("second"));
        Assert.Equal(palette, window.ReadTheme());

        Assert.False(Draw(dark: false, point: rectangle, press: true));
        Assert.False(Draw(dark: false, enabled: false));
        Assert.False(Draw(dark: false));
        Assert.False(Draw(dark: false, point: rectangle, press: false));

        bool Draw(bool dark, bool enabled = true, bool focus = false, int key = 0,
            CuiRectangle? point = null, bool? press = null)
        {
            Assert.True(window.NextFrame());
            if (point is { } target) window.MovePointer(target.X + target.Width / 2, target.Y + target.Height / 2);
            else window.MovePointer(-1, -1);
            if (press.HasValue) window.SetLeftButton(press.Value);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            bool activated = window.ThemeToggle("first", dark, enabled, focus);
            window.TrackLastControl("first");
            secondActivated = window.ThemeToggle("second", !dark, true, false);
            window.TrackLastControl("second");
            window.EndColumn();
            window.Present();
            return activated;
        }
    }
}
