using Fpas.Ui.Gui.Cui.Widgets.Editors;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks the managed limits of the native property inspector.</summary>
public sealed class CuiPropertyInspectorTests
{
    /// <summary>Rejects unsupported vectors and still accepts four-component values in the same frame.</summary>
    [GuiFact]
    public void EnforcesTheNativeVectorComponentLimit()
    {
        using var window = new CuiWindow("Property vectors", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var vector = new CuiProperty("Vector", 6, 0, [1, 2, 3, 4], null, 0, [], 0, 10, 1, 0, "", "", "", 256, 1);
        CuiPropertyGroup[] groups = [new("Transform", [vector with { Values = [1, 2, 3, 4, 5] }], true, false)];
        Assert.Throws<ArgumentException>("groups", () => window.PropertyInspector("properties", groups, 0, 1, 0, true));

        window.PropertyInspector("properties", [new("Transform", [vector], true, false)], 0, 1, 0, true);
        window.TrackLastControl("properties");
        window.Present();
        Assert.True(window.GetControlRectangle("properties").Height > 0);
    }
}
