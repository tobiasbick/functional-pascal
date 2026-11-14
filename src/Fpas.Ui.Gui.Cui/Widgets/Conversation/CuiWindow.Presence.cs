using Fpas.Ui.Gui.Cui.Interop.Conversation;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Conversation;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native group of users who are present.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="users">Users in strip order.</param>
    /// <param name="maximum">Number of visible avatars before overflow.</param>
    /// <param name="size">Avatar size in logical points.</param>
    public unsafe void Presence(string id, CuiPresenceUser[] users, int maximum, float size)
    {
        CheckFrame();
        CheckFeedbackNumber(size, nameof(size));
        if (maximum <= 0) throw new ArgumentOutOfRangeException(nameof(maximum));
        using var memory = new ChartMemory();
        var data = new NativePresenceUser[users.Length];
        for (int i = 0; i < data.Length; i++)
        {
            var user = users[i];
            Utf8.Validate(user.Name, nameof(users));
            Utf8.Validate(user.Avatar, nameof(users));
            CheckChartColor(user.Color);
            data[i] = new() { Name = memory.Text(user.Name), Avatar = memory.Text(Optional(user.Avatar)),
                Palette = user.Color.Palette, Rgba = user.Color.Rgba, Idle = user.Idle ? 1 : 0 };
        }
        fixed (NativePresenceUser* native = data)
            NativeMethods.Presence(context, Stable(id), native, data.Length, maximum, size);
    }
}
