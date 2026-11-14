using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Planning;

/// <summary>A native week appointment.</summary>
/// <param name="Title">Event title.</param>
/// <param name="Detail">Optional detail.</param>
/// <param name="Day">Day index.</param>
/// <param name="Start">Start hour.</param>
/// <param name="Finish">End hour.</param>
/// <param name="Tone">Semantic color.</param>
public readonly record struct CuiWeekEvent(string Title, string Detail, int Day, float Start, float Finish, int Tone);

/// <summary>Native edit proposal and tracked parts.</summary>
/// <param name="Events">Changed appointments, or null when untouched.</param>
/// <param name="Activated">Activated event index.</param>
/// <param name="Parts">Visible event availability.</param>
public readonly record struct CuiWeekResult(CuiWeekEvent[]? Events, int Activated, bool[] Parts);
