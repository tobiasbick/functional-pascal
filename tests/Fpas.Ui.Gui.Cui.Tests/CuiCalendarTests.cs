using Fpas.Ui.Gui.Cui.Widgets.Pickers;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks tracked calendar parts and focus across the native view modes.</summary>
public sealed class CuiCalendarTests
{
    /// <summary>Tracks title, page arrows, month and year choices through the managed ABI.</summary>
    /// <param name="renderer">Native renderer used to present the calendar.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void CalendarTracksItsThreeViews(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Calendar views", 500, 440, renderer, continuous: true);
        window.SetReducedMotion(true);
        DateOnly date = new(2026, 11, 14);
        var days = Draw(focus: 3);
        Assert.Equal(45, days.VisibleParts.Length);
        Assert.True(window.ControlFocused("part/-10"));
        var months = Draw(enter: true);
        Assert.Equal(15, months.VisibleParts.Length);
        Assert.Contains(-100 - 2026 * 12, months.VisibleParts);
        var years = Draw(enter: true);
        Assert.Equal(15, years.VisibleParts.Length);
        Assert.Contains(-200000 - 2016, years.VisibleParts);
        Draw(focus: 1);
        Assert.True(window.ControlFocused("part/-8"));
        var earlier = Draw(enter: true);
        Assert.Contains(-200000 - 2004, earlier.VisibleParts);
        Assert.Equal(date, earlier.Value);
        Assert.Equal(earlier.VisibleParts, Draw(track: false).VisibleParts);

        CuiCalendarResult Draw(int focus = 0, bool enter = false, bool track = true)
        {
            Assert.True(window.NextFrame());
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            var result = window.Calendar("calendar", date, true, focus, track ? part => "part/" + part : null);
            window.Present();
            return result;
        }
    }

    /// <summary>Returns the visible date-range controls with no tracking callback.</summary>
    [GuiFact]
    public void RangeCalendarReturnsVisibleDatesWithoutTracking()
    {
        using var window = new CuiWindow("Untracked date range", 640, 460, CuiRenderer.Software, continuous: true);
        int start = new DateOnly(2026, 11, 14).DayNumber;
        Assert.True(window.NextFrame());
        var result = window.RangeCalendar("range", start, start + 3, 1, true, trackPart: null);
        window.Present();
        Assert.Contains(start, result.VisibleParts);
    }

    /// <summary>Reports visible controls only while the single-date or date-range popup is open.</summary>
    [GuiTheory]
    [InlineData(false)]
    [InlineData(true)]
    public void DatePopupPartsDoNotRequireTracking(bool range)
    {
        using var window = new CuiWindow("Untracked date picker", 640, 550, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        DateOnly date = new(2026, 11, 14);
        Assert.Empty(Draw(focus: true, enter: false));
        Assert.Contains(date.DayNumber, Draw(focus: false, enter: true));

        int[] Draw(bool focus, bool enter)
        {
            Assert.True(window.NextFrame());
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            var parts = range
                ? window.RangePicker("date", date.DayNumber, date.DayNumber + 3, "", "", 0,
                    false, true, focus, trackPart: null).VisibleParts
                : window.DatePicker("date", date, "", "", 0, true, focus, trackPart: null).VisibleParts;
            window.Present();
            return parts;
        }
    }
}
