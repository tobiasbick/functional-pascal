namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks the native toggle group's caption and selection transport.</summary>
public sealed class CuiToggleGroupTests
{
    /// <summary>Preserves the last flag of a full group and leaves the caller's selection intact.</summary>
    [GuiFact]
    public void LastToggleKeepsItsLiteralCaptionAndProposesWithoutMutatingInput()
    {
        using var window = new CuiWindow("Toggle group flags", 1240, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.Null(Draw(["Literal caption with enough width", "B"], [true, false]));
        float plainWidth = window.GetControlRectangle("item/0").Width;
        Assert.Null(Draw(["##Literal caption with enough width", "B"], [true, false]));
        Assert.True(window.GetControlRectangle("item/0").Width > plainWidth);

        var labels = Enumerable.Repeat("A", 32).ToArray();
        var selected = new bool[32];
        selected[0] = selected[^1] = true;

        Assert.Null(Draw(labels, selected, focus: 32));
        Assert.True(window.ControlFocused("item/31"));
        bool[] proposal = Assert.IsType<bool[]>(Draw(labels, selected, key: 2));
        Assert.Equal(32, proposal.Length);
        Assert.True(proposal[0]);
        Assert.False(proposal[^1]);
        Assert.True(selected[^1]);
        Assert.Null(Draw(labels, selected, key: 2, enabled: false));
        Assert.False(window.ControlFocused("item/31"));

        bool[]? Draw(string[] captions, bool[] flags, int focus = 0, int key = 0, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (key != 0)
            {
                window.SetKey(key, true);
                window.SetKey(key, false);
            }
            window.BeginColumn();
            string[] tracking = ["group", .. Enumerable.Range(0, captions.Length).Select(index => $"item/{index}")];
            bool[]? result = window.ToggleGroup("flags", captions, null, flags, 0,
                exclusive: false, iconOnly: false, enabled, focus, tracking);
            window.EndColumn();
            window.Present();
            return result;
        }
    }
}
