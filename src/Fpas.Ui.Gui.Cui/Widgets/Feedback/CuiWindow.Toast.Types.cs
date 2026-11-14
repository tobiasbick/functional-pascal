using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Feedback;

/// <summary>Visible geometry and input state of part of a native toast.</summary>
/// <param name="Bounds">Finalized rectangle in logical points.</param>
/// <param name="Focused">Whether the part has native keyboard focus.</param>
/// <param name="Hovered">Whether the pointer is over the part.</param>
public sealed record CuiToastPart(CuiRectangle Bounds, bool Focused, bool Hovered);

/// <summary>The current native toast and any action delivered in this frame.</summary>
/// <param name="Active">Whether the notification is still pending or visible.</param>
/// <param name="Invoked">Whether the action was activated since the last poll.</param>
/// <param name="Card">Visible card, or null while it is hidden or dismissed.</param>
/// <param name="Action">Visible action button, or null.</param>
/// <param name="Dismiss">Visible close button, or null.</param>
public sealed record CuiToastFrame(bool Active, bool Invoked, CuiToastPart? Card, CuiToastPart? Action, CuiToastPart? Dismiss);
