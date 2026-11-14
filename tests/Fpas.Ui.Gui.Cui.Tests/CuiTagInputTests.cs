namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks native tag proposals that can contain more than one edit per frame.</summary>
public sealed class CuiTagInputTests
{
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
