using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Overlays;

/// <summary>Visible native tooltip in a completed frame.</summary>
/// <param name="Text">Rendered caption with its optional shortcut hint.</param>
/// <param name="Bounds">Finalized bounds in logical points.</param>
public sealed record CuiTooltipFrame(string Text, CuiRectangle Bounds);
