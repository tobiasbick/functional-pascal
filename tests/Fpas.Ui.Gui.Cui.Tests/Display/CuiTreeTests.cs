// [Documentation](../../../docs/ui/gui/data-display/tree.md)
using Fpas.Ui.Gui.Cui.Widgets.Collections;

namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks native tree activation and expansion through the managed adapter.</summary>
public sealed class CuiTreeTests
{
    /// <summary>Distinguishes row activation from an arrow-driven expansion.</summary>
    [GuiFact]
    public void BranchReportsActivationSeparatelyFromExpansion()
    {
        using var window = new CuiWindow("Native tree", 500, 400, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.Equal(new CuiTreeResult(false, false), Draw(open: false, focus: true));
        Assert.True(window.ControlFocused("branch"));
        Assert.Equal(new CuiTreeResult(true, false), Draw(open: false, key: 8));
        Assert.True(window.GetControlRectangle("leaf").Height > 0.0f);
        Assert.Equal(new CuiTreeResult(false, true), Draw(open: true, key: 2));

        CuiTreeResult Draw(bool open, bool focus = false, int key = 0)
        {
            Assert.True(window.NextFrame());
            if (key != 0)
            {
                window.SetKey(key, true);
                window.SetKey(key, false);
            }

            var result = window.BeginTree("branch", "Source", null, leaf: false,
                open: open, selected: false, enabled: true, focus: focus);
            window.TrackLastControl("branch");
            if (result.Open)
            {
                var leaf = window.BeginTree("leaf", "example.c", "", leaf: true,
                    open: true, selected: true, enabled: true, focus: false);
                Assert.False(leaf.Open);
                window.TrackLastControl("leaf");
                window.EndTree();
            }

            window.Present();
            return result;
        }
    }
}
