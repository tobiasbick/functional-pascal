using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Planning;

/// <summary>A native timeline task.</summary>
/// <param name="Name">Task name.</param>
/// <param name="Owner">Optional owner.</param>
/// <param name="Start">Start offset.</param>
/// <param name="Finish">End offset.</param>
/// <param name="Progress">Completed fraction.</param>
/// <param name="Tone">Semantic color.</param>
/// <param name="Dependency">Dependency index or minus one.</param>
/// <param name="Milestone">Milestone appearance.</param>
/// <param name="State">Native task state.</param>
public readonly record struct CuiGanttTask(string Name, string Owner, int Start, int Finish, float Progress, int Tone, int Dependency, bool Milestone, int State);

/// <summary>Native edit proposal and tracked parts.</summary>
/// <param name="Tasks">Changed task schedule, or null.</param>
/// <param name="Activated">Activated task index.</param>
/// <param name="Parts">Visible task availability.</param>
public readonly record struct CuiGanttResult(CuiGanttTask[]? Tasks, int Activated, bool[] Parts);
