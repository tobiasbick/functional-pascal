using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native chat message.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal body or Markdown source.</param>
    /// <param name="author">Optional sender name.</param>
    /// <param name="time">Optional timestamp.</param>
    /// <param name="status">Optional receipt.</param>
    /// <param name="avatar">Optional avatar image file.</param>
    /// <param name="flags">Outgoing, grouped, final and Markdown bits.</param>
    /// <param name="enabled">Whether the containing view accepts input.</param>
    public void ChatMessage(string id, string text, string author, string time, string status, string avatar, int flags, bool enabled)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(author, nameof(author));
        Utf8.Validate(time, nameof(time));
        Utf8.Validate(status, nameof(status));
        Utf8.Validate(avatar, nameof(avatar));
        NativeMethods.ChatMessage(context, Stable(id), text, Optional(author), Optional(time),
            Optional(status), Optional(avatar), flags, enabled ? 1 : 0);
    }
}
