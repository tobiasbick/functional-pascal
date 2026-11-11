using System.Buffers;
using System.Text;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Presentation and capacity of a native single-line editor.</summary>
/// <param name="Label">Optional caption above the field.</param>
/// <param name="Placeholder">Text displayed while empty.</param>
/// <param name="Icon">Optional leading glyph.</param>
/// <param name="Hint">Supporting text below the field.</param>
/// <param name="Error">Validation error that replaces the hint.</param>
/// <param name="Prefix">Muted text before the editable value.</param>
/// <param name="Suffix">Muted text after the editable value.</param>
/// <param name="Size">Native medium, small or large size ordinal.</param>
/// <param name="Password">Mask the text and show the native reveal button.</param>
/// <param name="Clearable">Show a clear button while editable and nonempty.</param>
/// <param name="ReadOnly">Allow focus and selection while preventing edits.</param>
/// <param name="MaxUtf8Bytes">Maximum value size excluding the terminator.</param>
public sealed record CuiTextFieldOptions(string Label, string Placeholder, string Icon, string Hint, string Error,
    string Prefix, string Suffix, int Size, bool Password, bool Clearable, bool ReadOnly, int MaxUtf8Bytes);

/// <summary>One frame's edit and native field affordances.</summary>
/// <param name="Value">Proposed value after native input.</param>
/// <param name="Submitted">Whether Enter submitted the field.</param>
/// <param name="ClearVisible">Whether the clear button was drawn.</param>
/// <param name="RevealVisible">Whether the reveal button was drawn.</param>
/// <param name="Revealed">Whether a password is currently revealed.</param>
public sealed record CuiTextFieldResult(string Value, bool Submitted, bool ClearVisible, bool RevealVisible, bool Revealed);

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's single-line editor using a detached UTF-8 buffer.</summary>
    /// <param name="id">Stable editor identity and snapshot key.</param>
    /// <param name="value">Current model-owned text.</param>
    /// <param name="options">Presentation and capacity.</param>
    /// <param name="enabled">Whether the editor and its actions accept input.</param>
    /// <param name="focus">Whether to request keyboard focus.</param>
    /// <param name="track">Whether to track the controls in this frame.</param>
    /// <param name="clearControl">Snapshot key of the optional clear button.</param>
    /// <param name="revealControl">Snapshot key of the optional password reveal button.</param>
    /// <returns>The resulting value, submission and affordance state.</returns>
    public unsafe CuiTextFieldResult TextField(string id, string value, CuiTextFieldOptions options,
        bool enabled, bool focus, bool track, string clearControl, string revealControl)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(options);
        string stable = Stable(id);
        Utf8.Validate(value, nameof(value));
        Utf8.Validate(options.Label, nameof(options.Label));
        Utf8.Validate(options.Placeholder, nameof(options.Placeholder));
        Utf8.Validate(options.Icon, nameof(options.Icon));
        Utf8.Validate(options.Hint, nameof(options.Hint));
        Utf8.Validate(options.Error, nameof(options.Error));
        Utf8.Validate(options.Prefix, nameof(options.Prefix));
        Utf8.Validate(options.Suffix, nameof(options.Suffix));
        if (options.Size is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(options.Size));
        if (options.MaxUtf8Bytes <= 0 || options.MaxUtf8Bytes >= Array.MaxLength
            || (options.Password && options.MaxUtf8Bytes > 341))
            throw new ArgumentOutOfRangeException(nameof(options.MaxUtf8Bytes));
        if (value.AsSpan().IndexOfAny('\r', '\n') >= 0 || Utf8.Encoding.GetByteCount(value) > options.MaxUtf8Bytes)
            throw new ArgumentException("The value must be a single line that fits the UTF-8 capacity.", nameof(value));

        int capacity = options.MaxUtf8Bytes + 1;
        byte[]? rented = capacity > 512 ? ArrayPool<byte>.Shared.Rent(capacity) : null;
        Span<byte> buffer = rented is null ? stackalloc byte[capacity] : rented.AsSpan(0, capacity);
        try
        {
            int length = Utf8.Encoding.GetBytes(value, buffer);
            buffer[length] = 0;
            int* nodes = stackalloc int[3];
            int flags;
            int revealed;
            fixed (byte* bytes = buffer)
                flags = NativeMethods.TextField(context, stable, bytes, (nuint)capacity,
                    Optional(options.Label), Optional(options.Placeholder), Optional(options.Icon),
                    Optional(options.Hint), Optional(options.Error), Optional(options.Prefix), Optional(options.Suffix),
                    options.Size, options.Password ? 1 : 0, options.Clearable ? 1 : 0,
                    options.ReadOnly ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, nodes, out revealed);
            if (track)
            {
                TrackControl(id, nodes[0]);
                if (nodes[1] >= 0) TrackControl(clearControl, nodes[1]);
                if (nodes[2] >= 0) TrackControl(revealControl, nodes[2]);
            }

            if ((flags & 1) != 0)
            {
                try { value = Utf8.Encoding.GetString(buffer[..buffer.IndexOf((byte)0)]); }
                catch (DecoderFallbackException) { flags = 0; }
            }
            return new(value, (flags & 2) != 0, nodes[1] >= 0, nodes[2] >= 0, revealed != 0);
        }
        finally
        {
            buffer.Clear();
            if (rented is not null) ArrayPool<byte>.Shared.Return(rented);
        }
    }

    private static string? Optional(string value) => value.Length == 0 ? null : value;
}
