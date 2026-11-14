using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks delivery of CUI's completion event through the managed result.</summary>
public sealed class CuiOneTimeCodeTests
{
    /// <summary>Reports native completion once and suppresses disabled input.</summary>
    [GuiFact]
    public void CompletionIsReturnedWithTheCode()
    {
        using var window = new CuiWindow("Native code completion", 480, 320, CuiRenderer.Software, continuous: true);
        string value = "";
        Assert.False(Draw(focus: true).Completed);
        var complete = Draw(text: "ABC123");
        Assert.Equal("ABC123", complete.Value);
        Assert.True(complete.Completed);
        Assert.False(Draw().Completed);
        Assert.False(Draw(enabled: false, text: "Z").Completed);
        Assert.Equal("ABC123", value);

        CuiOneTimeCodeResult Draw(bool focus = false, bool enabled = true, string? text = null)
        {
            Assert.True(window.NextFrame());
            if (text is not null) window.InputText(text);
            var result = window.OneTimeCode("code", value, 6, enabled, focus, track: true);
            value = result.Value;
            window.Present();
            return result;
        }
    }
}
