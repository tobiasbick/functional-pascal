// [Documentation](../../../docs/ui/gui/pickers/time-picker.md)
using Fpas.Ui.Gui.Cui.Widgets.Pickers;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks native sizing and focus for the controlled time picker.</summary>
public sealed class CuiTimePickerTests
{
    /// <summary>Passes each width policy into CUI before laying out labeled and unlabeled fields.</summary>
    [GuiFact]
    public void TimePickerUsesNativeExtents()
    {
        using var window = new CuiWindow("Time picker sizes", 640, 250, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        window.BeginColumn(12);
        Draw("content", null, 0, 0);
        Draw("empty label", "", 0, 0);
        Draw("fill", "", 1, 0);
        Draw("fixed", "Fixed ## caption", 2, 192.5f);
        Draw("percent", "Percent", 3, 40);
        window.EndColumn();
        window.Present();

        float available = window.Size.Width - 24;
        Assert.Equal(150, window.GetControlRectangle("content").Width, precision: 1);
        Assert.Equal(available, window.GetControlRectangle("empty label").Width, precision: 1);
        Assert.Equal(available, window.GetControlRectangle("fill").Width, precision: 1);
        Assert.Equal(192.5f, window.GetControlRectangle("fixed").Width, precision: 1);
        Assert.Equal(available * .4f, window.GetControlRectangle("percent").Width, precision: 1);

        void Draw(string id, string? label, int kind, float width) =>
            window.TimePicker(id, new TimeOnly(9, 33), label, kind, width, false, 5,
                true, false, choice => $"{id}/{choice}");
    }

    /// <summary>Closes an open picker and releases the trigger focus when disabled.</summary>
    [GuiFact]
    public void DisabledTimePickerClosesAndReleasesFocus()
    {
        using var window = new CuiWindow("Time picker focus", 640, 350, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        var value = new TimeOnly(9, 33);
        Draw(focus: true);
        Assert.True(window.ControlFocused("time"));
        var open = Draw(key: 2);
        Assert.Equal(value, open.Value);
        Assert.Contains(24 + 30, open.Choices);
        Assert.DoesNotContain(24 + 33, open.Choices);
        Assert.Empty(Draw(enabled: false).Choices);
        Assert.False(window.ControlFocused("time"));
        Assert.Empty(Draw(key: 2).Choices);

        CuiTimePickerResult Draw(bool enabled = true, bool focus = false, int key = 0)
        {
            Assert.True(window.NextFrame());
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            var result = window.TimePicker("time", value, null, 0, 0, false, 5,
                enabled, focus, choice => $"time/{choice}");
            window.Present();
            return result;
        }
    }
}
