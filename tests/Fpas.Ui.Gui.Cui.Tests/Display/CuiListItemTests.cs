// [Documentation](../../../docs/ui/gui/data-display/list-item.md)
namespace Fpas.Ui.Gui.Cui.Tests.Display;

/// <summary>Checks the native list item layout options.</summary>
public sealed class CuiListItemTests
{
    /// <summary>Distinguishes an absent subtitle from a reserved second line.</summary>
    [GuiFact]
    public void BlankSubtitleUsesTwoLineRow()
    {
        using var window = new CuiWindow("List item options", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginColumn();
        Draw("single-line", null);
        Draw("blank-subtitle", "");
        Draw("visible-subtitle", "Tomorrow at 10:00");
        window.EndColumn();
        window.Present();
        float singleLine = window.GetControlRectangle("single-line").Height;
        float blankSubtitle = window.GetControlRectangle("blank-subtitle").Height;
        float visibleSubtitle = window.GetControlRectangle("visible-subtitle").Height;
        Assert.True(blankSubtitle >= 52f && blankSubtitle > singleLine);
        Assert.True(visibleSubtitle >= blankSubtitle);

        void Draw(string id, string? subtitle)
        {
            window.ListItem(id, "Review", subtitle, null, null, null, selected: false, enabled: true, focus: false);
            window.TrackLastControl(id);
        }
    }
}
