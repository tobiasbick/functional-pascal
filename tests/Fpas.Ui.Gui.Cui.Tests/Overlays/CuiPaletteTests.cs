// [Documentation](../../../docs/ui/gui/overlays/command-palette.md)
using Fpas.Ui.Gui.Cui.Widgets.Overlays;

namespace Fpas.Ui.Gui.Cui.Tests.Overlays;

/// <summary>Checks native palette option marshalling.</summary>
public sealed class CuiPaletteTests
{
    /// <summary>Only a supplied group label reserves space ahead of the first command.</summary>
    [GuiFact]
    public void BlankGroupReservesHeading()
    {
        using var window = new CuiWindow("Palette options", 640, 480, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.True(CommandTop("") > CommandTop(null));

        float CommandTop(string? group)
        {
            Assert.True(window.NextFrame());
            var result = window.CommandPalette("palette", open: true, enabled: true,
                [new CuiPaletteCommand("Open project", null, group, null)], "query", _ => "command");
            Assert.True(result.Open);
            window.Present();
            return window.GetControlRectangle("command").Y;
        }
    }
}
