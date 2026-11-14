using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Conversation;

/// <summary>A visible user in a native presence strip.</summary>
/// <param name="Name">Tooltip and initials source.</param>
/// <param name="Color">Ring color.</param>
/// <param name="Avatar">Optional avatar image file.</param>
/// <param name="Idle">Whether the user is away.</param>
public readonly record struct CuiPresenceUser(string Name, CuiChartColor Color, string Avatar, bool Idle);
