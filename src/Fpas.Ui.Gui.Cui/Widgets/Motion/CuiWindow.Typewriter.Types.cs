using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Motion;

/// <summary>Current typewriter progress at the end of native presentation.</summary>
/// <param name="Done">Whether all characters are visible.</param>
/// <param name="Completed">Whether this frame completed the current text or replay.</param>
public readonly record struct CuiTypewriterResult(bool Done, bool Completed);
