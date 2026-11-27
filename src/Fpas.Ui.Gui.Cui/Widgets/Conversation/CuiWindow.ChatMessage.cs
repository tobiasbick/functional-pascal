// [Documentation](../../../../docs/ui/gui/social-and-media/chat-message.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Display;
using System.Runtime.InteropServices;

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
    /// <param name="flags">Outgoing, grouped, final, Markdown and Markdown input bits.</param>
    /// <param name="enabled">Whether the containing view accepts input.</param>
    /// <returns>Native Markdown link, hover and task observations.</returns>
    public CuiMarkdownEvent ChatMessage(string id, string text, string? author, string? time,
        string? status, string avatar, int flags, bool enabled)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        if (author is not null) Utf8.Validate(author, nameof(author));
        if (time is not null) Utf8.Validate(time, nameof(time));
        if (status is not null) Utf8.Validate(status, nameof(status));
        Utf8.Validate(avatar, nameof(avatar));
        nint link = NativeMethods.ChatMessage(context, Stable(id), text, author, time,
            status, Optional(avatar), flags, enabled ? 1 : 0, out int offset, out int line, out int check,
            out nint hovered, out int hoverChanged);
        return new(Marshal.PtrToStringUTF8(link), offset, line, check != 0,
            Marshal.PtrToStringUTF8(hovered), hoverChanged != 0);
    }
}
