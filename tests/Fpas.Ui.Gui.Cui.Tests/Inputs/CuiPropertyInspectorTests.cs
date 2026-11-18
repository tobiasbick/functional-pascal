using Fpas.Ui.Gui.Cui.Widgets.Editors;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

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

    /// <summary>Rejects text that cannot be represented by the native UTF-8 field.</summary>
    [GuiTheory]
    [InlineData("left\0right", 32)]
    [InlineData("é", 2)]
    [InlineData("", 0)]
    public void ValidatesTheNativeTextBuffer(string text, int capacity)
    {
        using var window = new CuiWindow("Property text", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var property = new CuiProperty("Text", 2, 0, [], text, 0, [], 0, 0, 0, -1, "", "", "", capacity, 1);
        Assert.Throws<ArgumentException>("groups", () => Draw(property));
        Draw(property with { Text = "é", Capacity = 3 });
        window.Present();

        void Draw(CuiProperty row) =>
            window.PropertyInspector("properties", [new("Details", [row], true, false)], 0, 1, 0, true);
    }
}
