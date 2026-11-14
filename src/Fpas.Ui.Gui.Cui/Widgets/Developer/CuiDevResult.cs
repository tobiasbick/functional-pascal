namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>Values proposed by a native developer widget.</summary>
/// <param name="Index">Index supplied to CUI.</param>
/// <param name="Action">Action supplied to CUI.</param>
/// <param name="Hover">Hover supplied to CUI.</param>
/// <param name="Count">Count supplied to CUI.</param>
/// <param name="Flags">Flags supplied to CUI.</param>
/// <param name="X">X supplied to CUI.</param>
/// <param name="Y">Y supplied to CUI.</param>
/// <param name="Z">Z supplied to CUI.</param>
/// <param name="A">A supplied to CUI.</param>
/// <param name="B">B supplied to CUI.</param>
public readonly record struct CuiDevResult(int Index, int Action, int Hover, int Count, int Flags, double X, double Y, double Z, double A, double B);
