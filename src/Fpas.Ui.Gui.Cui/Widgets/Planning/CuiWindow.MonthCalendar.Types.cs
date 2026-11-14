using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Planning;

/// <summary>A native month appointment.</summary>
/// <param name="Title">Event title.</param>
/// <param name="Start">First day number.</param>
/// <param name="Finish">Last day number or minus one.</param>
/// <param name="Time">Hour of a timed event.</param>
/// <param name="AllDay">Full-day appearance.</param>
/// <param name="Tone">Semantic color.</param>
public readonly record struct CuiMonthEvent(string Title, int Start, int Finish, float Time, bool AllDay, int Tone);

/// <summary>Native edit proposal and tracked parts.</summary>
/// <param name="Month">Proposed month start day number.</param>
/// <param name="Selected">Selected day number or minus one.</param>
/// <param name="Activated">Activated event index.</param>
/// <param name="Flags">Native day and month changes.</param>
/// <param name="Parts">Visible part availability.</param>
public readonly record struct CuiMonthResult(int Month, int Selected, int Activated, int Flags, bool[] Parts);
