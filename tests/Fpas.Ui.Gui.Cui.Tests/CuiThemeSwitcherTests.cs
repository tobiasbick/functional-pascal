namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks controlled native appearance choices without palette side effects.</summary>
public sealed class CuiThemeSwitcherTests
{
    /// <summary>Preserves custom colors, stable segment identities and native keyboard behavior.</summary>
    [GuiFact]
    public void ChoicesRemainControlledAndDoNotReplaceTheHostPalette()
    {
        using var window = new CuiWindow("Theme switcher", 480, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        uint[] palette = window.ReadTheme();
        palette[0] = 0xff352719;
        palette[4] = 0xffc7d9eb;
        window.SetTheme(true, 2, palette);
        Assert.Equal(-1, Draw(1, focus: 1));
        Assert.Equal(30, window.GetControlRectangle("group").Height);
        Assert.Equal(24, window.GetControlRectangle("light").Width);
        Assert.True(window.ControlFocused("light"));
        Assert.Equal(0, Draw(1, key: 2));
        Assert.Equal(palette, window.ReadTheme());
        Assert.Equal(-1, Draw(1));
        Assert.Equal(-1, Draw(1, key: 8));
        Assert.True(window.ControlFocused("dark"));
        Assert.Equal(2, Draw(1, key: 8));
        Assert.True(window.ControlFocused("system"));
        Assert.Equal(-1, Draw(2, key: 8));
        Assert.Equal(-1, Draw(2, key: 6));
        Assert.Equal(1, Draw(2, key: 7));
        Assert.True(window.ControlFocused("dark"));
        Assert.Equal(palette, window.ReadTheme());
        Assert.Equal(-1, Draw(1, enabled: false, key: 7));
        Assert.False(window.ControlFocused("dark"));
        Assert.Equal(-1, Draw(1, focus: 2));
        Assert.Equal(-1, Draw(1, withSystem: false));
        Assert.True(window.ControlFocused("dark"));
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("system"));
        Assert.Equal(palette, window.ReadTheme());
        Assert.Equal(78 - 24, window.GetControlRectangle("group").Width);

        int Draw(int selected, int key = 0, int focus = 0, bool enabled = true, bool withSystem = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            int proposal = window.ThemeSwitcher("appearance", selected, withSystem, enabled, focus,
                withSystem ? ["group", "light", "dark", "system"] : ["group", "light", "dark"]);
            window.EndColumn();
            window.Present();
            return proposal;
        }
    }

    /// <summary>Uses stable scopes for multiple selectors and routes real pointer releases to the hit segment.</summary>
    [GuiFact]
    public void InstancesAndPointerChoicesAreIndependent()
    {
        using var window = new CuiWindow("Two theme switchers", 480, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        CuiRectangle? target = null;
        Draw();
        target = window.GetControlRectangle("second/system");
        Draw(button: true);
        Assert.Equal((-1, 2), Draw(button: false));
        Assert.Equal((-1, -1), Draw());
        Assert.True(window.ControlFocused("second/system"));

        (int First, int Second) Draw(bool? button = null)
        {
            Assert.True(window.NextFrame());
            if (target is { } hit) window.MovePointer(hit.X + hit.Width / 2, hit.Y + hit.Height / 2);
            if (button.HasValue) window.SetLeftButton(button.Value);
            window.BeginColumn();
            int first = window.ThemeSwitcher("first", 0, true, true, 0,
                ["first", "first/light", "first/dark", "first/system"]);
            int second = window.ThemeSwitcher("second", 0, true, true, 0,
                ["second", "second/light", "second/dark", "second/system"]);
            window.EndColumn(); window.Present();
            return (first, second);
        }
    }

    /// <summary>Rejects unavailable modes, invalid focus targets and incomplete part tracking.</summary>
    [GuiFact]
    public void ChoiceAndTrackingOptionsAreValidated()
    {
        using var window = new CuiWindow("Theme validation", 480, 320, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ThemeSwitcher("id", 2, false, true, 0, null));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ThemeSwitcher("id", -1, true, true, 0, null));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ThemeSwitcher("id", 3, true, true, 0, null));
        Assert.Throws<ArgumentOutOfRangeException>(() => window.ThemeSwitcher("id", 0, false, true, 3, null));
        Assert.Throws<ArgumentException>(() => window.ThemeSwitcher("id", 0, true, true, 0, ["only-group"]));
    }
}
