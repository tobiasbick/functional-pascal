// [Documentation](../../../docs/ui/gui/inputs/tag-input.md)
namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks native tag proposals that can contain more than one edit per frame.</summary>
public sealed class CuiTagInputTests
{
    /// <summary>Applies native width to the whole tag field, including its caption.</summary>
    [GuiTheory]
    [InlineData(0, 0f, 432f)]
    [InlineData(1, 0f, 432f)]
    [InlineData(2, 152f, 152f)]
    [InlineData(3, 50f, 216f)]
    public void TagInputUsesNativeWidth(int kind, float width, float expected)
    {
        using var window = new CuiWindow("Tag width", 480, 320, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginColumn();
        window.TagInput("captioned", ["First"], "Topics", null, 64, 255,
            enabled: true, focus: false, removeControl: null, widthKind: kind, width: width);
        window.TrackLastControl("captioned-field");
        window.TagInput("plain", [], "", "", 64, 255,
            enabled: true, focus: false, removeControl: null, widthKind: kind, width: width);
        window.TrackLastControl("plain-field");
        window.EndColumn();
        window.Present();

        Assert.Equal(expected, window.GetControlRectangle("captioned-field").Width);
        Assert.Equal(expected, window.GetControlRectangle("plain-field").Width);
    }

    /// <summary>Retains only the allowed tags when one input event contains multiple separators.</summary>
    [GuiFact]
    public void CommaSeparatedInputHonorsTheTagLimit()
    {
        using var window = new CuiWindow("Bounded tags", 480, 320, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        string[] tags = [];
        window.TagInput("tags", tags, "", "", 1, 255, enabled: true, focus: true, removeControl: null);
        window.Present();

        Assert.True(window.NextFrame());
        window.InputText("one,two,");
        var result = window.TagInput("tags", tags, "", "", 1, 255, enabled: true, focus: false, removeControl: null);
        window.Present();
        Assert.Equal(["one"], result.Tags);
        Assert.True(window.NextFrame());
        var drawn = window.TagInput("tags", result.Tags, "", "", 1, 255,
            enabled: true, focus: false, removeControl: null);
        window.Present();
        Assert.Equal(1, drawn.RemoveCount);
    }
}
