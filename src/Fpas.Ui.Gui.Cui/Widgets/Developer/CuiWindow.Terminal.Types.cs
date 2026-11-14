using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>Terminal input from the native widget.</summary>
/// <param name="Command">Submitted line when present.</param>
/// <param name="Interrupt">Whether Ctrl+C requested an interrupt.</param>
public sealed record CuiTerminalEvent(string? Command, bool Interrupt);
