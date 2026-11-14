using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Feedback;

/// <summary>Actions and presence of the controls within a native banner.</summary>
/// <param name="Activated">Whether the action was clicked.</param>
/// <param name="Dismissed">Whether the close button was clicked.</param>
/// <param name="Visible">Whether the banner still occupies space.</param>
/// <param name="HasAction">Whether the action button was drawn.</param>
/// <param name="HasDismiss">Whether the dismiss button was drawn.</param>
public readonly record struct CuiBannerResult(bool Activated, bool Dismissed, bool Visible, bool HasAction, bool HasDismiss);
