namespace Fpas.Ui.Gui.Cui.Resources;

/// <summary>Native calendar/file-browser captions; null entries retain CUI's English defaults.</summary>
/// <param name="Months">Twelve full month names, January first; null uses defaults.</param>
/// <param name="ShortMonths">Twelve short month names; null uses defaults.</param>
/// <param name="Weekdays">Seven full weekday names, Monday first; null uses defaults.</param>
/// <param name="ShortWeekdays">Seven short weekday names; null uses defaults.</param>
/// <param name="MinimalWeekdays">Seven minimal weekday names; null uses defaults.</param>
/// <param name="Today">Today-button caption; null uses English.</param>
public sealed record CuiStrings(string?[]? Months = null, string?[]? ShortMonths = null,
    string?[]? Weekdays = null, string?[]? ShortWeekdays = null, string?[]? MinimalWeekdays = null,
    string? Today = null);
