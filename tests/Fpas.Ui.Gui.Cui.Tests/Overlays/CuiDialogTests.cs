// [Documentation](../../../docs/ui/gui/overlays/dialog.md)
namespace Fpas.Ui.Gui.Cui.Tests.Overlays;

/// <summary>Checks native dialog options across managed marshalling.</summary>
public sealed class CuiDialogTests
{
    /// <summary>Preserves the native height of a supplied blank description.</summary>
    [GuiFact]
    public void EmptyDescriptionReservesSpace()
    {
        using var window = new CuiWindow("Dialog supporting text", 500, 400, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        float omitted = Present(null);
        float blank = Present("");

        Assert.True(omitted > 0.0f);
        Assert.True(blank > omitted);

        float Present(string? description)
        {
            Assert.True(window.NextFrame());
            Assert.Equal(1, window.BeginDialog("dialog", "Dialog", description, 0.0f,
                open: true, enabled: true, persistent: false, closeButton: false, focus: false, closeControl: ""));
            window.EndDialog();
            window.Present();
            return window.GetControlRectangle("dialog").Height;
        }
    }
}
