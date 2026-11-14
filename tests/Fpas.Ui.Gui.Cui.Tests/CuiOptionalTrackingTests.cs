using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks visible widget parts with optional inspection tracking disabled.</summary>
public sealed class CuiOptionalTrackingTests
{
    /// <summary>Reports the rendered action of the first unfinished checklist step.</summary>
    [GuiFact]
    public void ChecklistReturnsActionsWithoutTracking()
    {
        using var window = new CuiWindow("Checklist actions", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        var result = window.Checklist("checklist", "", "", ["Create", "Invite"], ["", ""], ["", "Send"],
            [1, 0], enabled: true, focus: 0, trackPart: null);
        window.Present();
        Assert.Equal([1], result.VisibleActions);
    }

    /// <summary>Reports the original option indices when an untracked popup opens.</summary>
    [GuiFact]
    public void SelectReturnsOptionsWithoutTracking()
    {
        using var window = new CuiWindow("Select choices", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.Empty(Draw(focus: true, enter: false).VisibleOptions);
        var open = Draw(focus: false, enter: true);
        Assert.True(open.Open);
        Assert.Equal([0, 1], open.VisibleOptions);

        CuiSelectResult Draw(bool focus, bool enter)
        {
            Assert.True(window.NextFrame());
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            var result = window.Select("select", ["One", "Two"], ["", ""], -1, "", "",
                searchable: false, enabled: true, focus, trackOption: null);
            window.Present();
            return result;
        }
    }
}
