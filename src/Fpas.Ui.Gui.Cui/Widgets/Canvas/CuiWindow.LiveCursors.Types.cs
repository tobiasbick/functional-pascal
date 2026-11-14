using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>Native data for LiveCursors.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Tone">Tone supplied to CUI.</param>
/// <param name="X">X supplied to CUI.</param>
/// <param name="Y">Y supplied to CUI.</param>
/// <param name="Says">Says supplied to CUI.</param>
/// <param name="Speaking">Speaking supplied to CUI.</param>
/// <param name="Away">Away supplied to CUI.</param>
/// <param name="Rgba">Rgba supplied to CUI.</param>
public sealed record CuiLivePerson(string Name, int Tone, float X, float Y, string? Says, bool Speaking, bool Away, uint Rgba);
