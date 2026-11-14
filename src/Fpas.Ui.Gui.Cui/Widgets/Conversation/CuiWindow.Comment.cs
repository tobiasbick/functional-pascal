using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native comment and reports its like and reply actions.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="author">Author name.</param>
    /// <param name="text">Literal comment.</param>
    /// <param name="time">Optional timestamp.</param>
    /// <param name="depth">Reply indentation level.</param>
    /// <param name="badge">Optional author badge.</param>
    /// <param name="flags">Edited and enabled bits.</param>
    /// <param name="focus">Like or Reply index, or minus one.</param>
    /// <param name="likes">Like count, updated on activation.</param>
    /// <param name="liked">Current and proposed like flag.</param>
    /// <param name="likeId">Optional snapshot identity for Like.</param>
    /// <param name="replyId">Optional snapshot identity for Reply.</param>
    /// <returns>Native like and reply bits.</returns>
    public unsafe int Comment(string id, string author, string text, string time, int depth, string badge, int flags, int focus, ref int likes, ref int liked, string? likeId, string? replyId)
    {
        CheckFrame();
        Utf8.Validate(author, nameof(author));
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(time, nameof(time));
        Utf8.Validate(badge, nameof(badge));
        if (depth < 0 || likes < liked || likes == int.MaxValue || liked is < 0 or > 1)
            throw new ArgumentOutOfRangeException(nameof(likes));
        int* parts = stackalloc int[2];
        int result = NativeMethods.Comment(context, Stable(id), author, text, Optional(time), depth,
            Optional(badge), flags, focus, ref likes, ref liked, parts);
        if (likeId is not null && parts[0] >= 0) TrackControl(likeId, parts[0]);
        if (replyId is not null && parts[1] >= 0) TrackControl(replyId, parts[1]);
        return result;
    }
}
