using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native tag input using the model's current tags.</summary>
    /// <param name="id">Stable editor identity.</param>
    /// <param name="tags">Ordered nonempty tags without commas or surrounding spaces.</param>
    /// <param name="label">Optional caption.</param>
    /// <param name="placeholder">Text displayed when the list is empty.</param>
    /// <param name="maxTags">Maximum tag count, from 1 to 64.</param>
    /// <param name="maxUtf8Bytes">Maximum packed byte count, from 1 to 2047.</param>
    /// <param name="enabled">Whether edits are allowed.</param>
    /// <param name="focus">Whether to focus the draft editor.</param>
    /// <param name="removeControl">Maps tag positions to tracking IDs. Null disables tracking.</param>
    /// <returns>The current tags and visible remove actions.</returns>
    public unsafe CuiTagInputResult TagInput(string id, string[] tags, string label, string placeholder,
        int maxTags, int maxUtf8Bytes, bool enabled, bool focus, Func<int, string>? removeControl)
    {
        CheckFrame();
        string stable = Stable(id);
        ArgumentNullException.ThrowIfNull(tags);
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(placeholder, nameof(placeholder));
        if (maxTags is < 1 or > 64) throw new ArgumentOutOfRangeException(nameof(maxTags));
        if (maxUtf8Bytes is < 1 or > 2047) throw new ArgumentOutOfRangeException(nameof(maxUtf8Bytes));
        if (tags.Length > maxTags) throw new ArgumentException("Too many tags.", nameof(tags));
        foreach (string tag in tags)
        {
            Utf8.Validate(tag, nameof(tags));
            if (tag.Length == 0 || tag[0] == ' ' || tag[^1] == ' ' || tag.AsSpan().ContainsAny(',', '\r', '\n'))
                throw new ArgumentException("Expected nonempty single-line tags without commas or surrounding spaces.", nameof(tags));
        }
        string current = string.Join(", ", tags);
        if (Utf8.Encoding.GetByteCount(current) > maxUtf8Bytes)
            throw new ArgumentException("The tag list exceeds its UTF-8 capacity.", nameof(tags));

        Span<byte> buffer = stackalloc byte[maxUtf8Bytes + 1];
        buffer[Utf8.Encoding.GetBytes(current, buffer)] = 0;
        int* nodes = stackalloc int[tags.Length + 1];
        fixed (byte* bytes = buffer)
            NativeMethods.TagInput(context, stable, bytes, (nuint)buffer.Length, Optional(label), Optional(placeholder),
                maxTags, enabled ? 1 : 0, focus ? 1 : 0, tags.Length, nodes);

        int removals = 0;
        if (removeControl is not null)
        {
            TrackControl(id, nodes[0]);
            for (int index = 0; index < tags.Length && nodes[index + 1] >= 0; index++)
            {
                TrackControl(removeControl(index), nodes[index + 1]);
                removals++;
            }
        }

        string proposed = Utf8.Encoding.GetString(buffer[..buffer.IndexOf((byte)0)]);
        if (!enabled || proposed == current) return new(tags, current, removals);
        string[] updated = proposed.Length == 0 ? [] : proposed.Split(", ", StringSplitOptions.None);
        return new(proposed == current ? tags : updated, proposed, removals);
    }
}
