// [Documentation](../../../docs/ui/gui/selection/radio-cards.md)
using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks CUI's optional radio content across managed marshalling.</summary>
public sealed class CuiRadioTests
{
    /// <summary>Preserves the layout distinction between absent content and blank content.</summary>
    [GuiFact]
    public void BlankContentKeepsItsNativeSpace()
    {
        using var window = new CuiWindow("Radio content", 500, 400, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginColumn();
        window.RadioButton("omitted-row", "Save", null, false, true, false);
        window.TrackLastControl("omitted-row");
        window.RadioButton("blank-row", "Save", "", false, true, false);
        window.TrackLastControl("blank-row");
        window.RadioCards("omitted-cards", [new CuiRadioCard("Save", null, null, null)], -1, 1, true, 0, null);
        window.TrackLastControl("omitted-cards");
        window.RadioCards("blank-cards", [new CuiRadioCard("Save", "", "", "")], -1, 1, true, 0, null);
        window.TrackLastControl("blank-cards");
        window.EndColumn();
        window.Present();

        Assert.True(window.GetControlRectangle("blank-row").Height > window.GetControlRectangle("omitted-row").Height);
        Assert.True(window.GetControlRectangle("blank-cards").Height > window.GetControlRectangle("omitted-cards").Height);
    }
}
