// [Documentation](../../../docs/ui/gui/inputs/README.md)
using Fpas.Ui.Gui.Cui.Widgets.Feedback;
using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks visible widget parts with optional inspection tracking disabled.</summary>
public sealed class CuiOptionalTrackingTests
{
    /// <summary>Preserves empty code block captions and reports the native copy button independently of tracking.</summary>
    [GuiFact]
    public void CodeBlockDistinguishesMissingAndEmptyHeaderText()
    {
        using var window = new CuiWindow("Code block headers", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        const string source = "Example";
        Assert.False(window.CodeBlock("default", source, null, null, 0, 0, 0, 0, 4, copyId: null));
        Assert.True(window.CodeBlock("empty-title", source, null, "", 0, 0, 0, 0, 4, copyId: null));
        Assert.True(window.CodeBlock("empty-language", source, "", null, 0, 0, 0, 0, 4, copyId: null));
        Assert.False(window.CodeBlock("hidden", source, "", "", 0, 0, 0, 0, 6, copyId: null));
        window.Present();
    }

    /// <summary>Reports the rendered action of the first unfinished checklist step.</summary>
    [GuiFact]
    public void ChecklistReturnsActionsWithoutTracking()
    {
        using var window = new CuiWindow("Checklist actions", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.Equal([1], Draw("Send#", focus: true, enter: false).VisibleActions);
        var activated = Draw("", focus: false, enter: true);
        Assert.Equal([1], activated.VisibleActions);
        Assert.Equal(1, activated.Action);
        Assert.Empty(Draw(null, focus: false, enter: false).VisibleActions);

        CuiChecklistResult Draw(string? caption, bool focus, bool enter)
        {
            Assert.True(window.NextFrame());
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            var result = window.Checklist("checklist", null, null, ["Create", "Invite"], [null, null], [null, caption],
                [1, 0], enabled: true, focus: focus ? 4 : 0, trackPart: null);
            window.Present();
            return result;
        }
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
