// [Documentation](../../../docs/ui/gui/editing-tools/property-inspector.md)
using Fpas.Ui.Gui.Cui.Widgets.Editors;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Tests.Support;

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

    /// <summary>A custom number format takes precedence over the fixed digit count.</summary>
    [GuiFact]
    public void DrawsCustomNumericFormats()
    {
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Property formats", 420, 240, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        var property = new CuiProperty("Number", 0, 0, [12.5f], null, 0, [], 0, 0, 0, 0, "", "", "", 256, 1);
        uint[] standard = Draw("standard.png", property);
        uint[] custom = Draw("custom.png", property with { Format = new CuiChartFormat(2, "USD ") });
        uint[] differentDigits = Draw("digits.png", property with { Digits = 7, Format = new CuiChartFormat(2, "USD ") });

        Assert.False(standard.SequenceEqual(custom));
        Assert.Equal(custom, differentDigits);

        uint[] Draw(string name, CuiProperty row)
        {
            for (int frame = 0; frame < 2; frame++)
            {
                Assert.True(window.NextFrame());
                window.PropertyInspector("properties", [new("Details", [row], true, false)], 0, 1, 0, true);
                window.Present();
            }
            string path = directory.File(name);
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, 420, 240);
        }
    }
}
