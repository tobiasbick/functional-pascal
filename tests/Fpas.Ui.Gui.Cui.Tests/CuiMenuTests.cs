using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks native dropdown commands, keyed captions and lifecycle.</summary>
public sealed class CuiMenuTests
{
    /// <summary>Preserves native disabled/highlight behavior, literal text, stable keys and dismissal.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void CommandsUseNativeNavigationAndLiteralIdentities(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Dropdown commands", 760, 540, renderer, continuous: true);
        window.SetReducedMotion(true);
        CuiMenuEntry[] entries = [new(CuiMenuEntryKind.Label, "", "Project"),
            new(CuiMenuEntryKind.Action, "disabled", "Disabled", Enabled: false),
            new(CuiMenuEntryKind.Separator, "", ""),
            new(CuiMenuEntryKind.Action, "rename", "Rename ## literally", Shortcut: "F2", Checked: true),
            new(CuiMenuEntryKind.Action, "delete", "Delete", Danger: true)];
        Draw(focus: true);
        Assert.True(window.ControlFocused("trigger"));
        Assert.True(Draw(key: 10).Open);
        Assert.Equal(1, Draw(key: 10).Highlighted);
        Assert.Equal(-1, Draw(key: 2).Choice);
        Assert.True(Draw().Open);
        Assert.Equal(3, Draw(key: 10).Highlighted);
        var command = window.GetControlRectangle("rename");
        Assert.True(command.Width >= 168);
        Assert.Equal(3, Draw(key: 2).Choice);
        Assert.False(Draw().Open);
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("rename"));
        Assert.True(Draw(key: 10).Open);
        entries[3] = entries[3] with { Label = "Renamed ### literally" };
        Assert.True(Draw(label: "Updated ## title").Open);
        Assert.True(window.ControlFocused("trigger"));
        Assert.False(Draw(key: 3).Open);
        Assert.True(Draw(key: 10).Open);
        Assert.False(Draw(enabled: false).Open);
        Assert.False(window.ControlFocused("trigger"));
        Draw(iconOnly: true, focus: true);
        Assert.Equal(36, window.GetControlRectangle("trigger").Width);
        Assert.True(Draw(iconOnly: true, key: 10).Open);
        Assert.False(Draw(iconOnly: true, key: 3).Open);
        Assert.True(Draw(iconOnly: true, key: 10).Open);
        Assert.False(Draw(iconOnly: true, outside: true).Open);
        Draw(iconOnly: true, focus: true);
        Assert.True(Draw(iconOnly: true, key: 10).Open);
        Assert.True(window.NextFrame());
        window.BeginColumn();
        window.EndColumn();
        window.Present();
        Assert.False(Draw(iconOnly: true).Open);

        CuiMenuResult Draw(bool focus = false, int key = 0, bool enabled = true, bool iconOnly = false,
            string label = "Options", bool outside = false)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(outside ? 750 : -1, outside ? 530 : -1);
            window.SetLeftButton(outside);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            var result = window.Menu("commands", label, "", entries, 0, 0, 0, iconOnly, enabled,
                focus, ["trigger", "header", "disabled", "separator", "rename", "delete"]);
            window.EndColumn();
            window.Present();
            return result;
        }
    }
}
