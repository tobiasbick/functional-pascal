using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Feedback;

/// <summary>Native checklist proposals and action availability.</summary>
/// <param name="Action">Index of the invoked step action, or minus one.</param>
/// <param name="Completed">Proposed completion flags.</param>
/// <param name="VisibleActions">Indices of rendered step action buttons.</param>
public readonly record struct CuiChecklistResult(int Action, int[] Completed, int[] VisibleActions);
