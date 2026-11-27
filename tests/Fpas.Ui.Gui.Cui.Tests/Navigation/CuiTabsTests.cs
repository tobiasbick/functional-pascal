// [Documentation](../../../docs/ui/gui/navigation/tabs.md)
namespace Fpas.Ui.Gui.Cui.Tests.Navigation;

/// <summary>Checks native tab sizing and input with icon-only and labeled choices.</summary>
public sealed class CuiTabsTests
{
    /// <summary>Uses the native square icon-only tab in both appearances.</summary>
    [GuiTheory]
    [InlineData(0)]
    [InlineData(1)]
    public void IconOnlyTabKeepsNativeSizeAndActivation(int variant)
    {
        using var window = new CuiWindow("Tab icons", 520, 320, CuiRenderer.Software, continuous: true);
        string[] labels = ["", "Visible caption"];
        string[] icons = ["\uE19F", "\uE19F"];
        string[] track = ["strip", "icon", "caption"];
        Assert.True(window.NextFrame());
        Assert.Equal(-1, window.Tabs("tabs", labels, icons, 1, variant,
            fullWidth: false, enabled: true, focus: 1, trackingIds: track));
        window.Present();

        var icon = window.GetControlRectangle("icon");
        Assert.True(icon.Width > 0);
        Assert.Equal(icon.Height, icon.Width);
        Assert.True(window.GetControlRectangle("caption").Width > icon.Width);
        Assert.True(window.ControlFocused("icon"));

        Assert.True(window.NextFrame());
        window.SetKey(2, true);
        window.SetKey(2, false);
        Assert.Equal(0, window.Tabs("tabs", labels, icons, 1, variant,
            fullWidth: false, enabled: true, focus: 0, trackingIds: track));
        window.Present();
    }
}
