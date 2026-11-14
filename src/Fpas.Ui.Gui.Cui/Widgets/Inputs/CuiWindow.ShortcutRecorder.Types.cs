using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>Native key and modifier values for a recorded shortcut.</summary>
/// <param name="Key">CUI key ordinal, zero for no shortcut.</param>
/// <param name="Modifiers">Shift one, Control two, Alt four and Super eight.</param>
public readonly record struct CuiKeyCombination(int Key, int Modifiers);

/// <summary>Native recorder proposal and the clear button drawn in this frame.</summary>
/// <param name="Value">Proposed key and modifier combination.</param>
/// <param name="ClearVisible">Whether the native clear action is present.</param>
public sealed record CuiShortcutRecorderResult(CuiKeyCombination Value, bool ClearVisible);
