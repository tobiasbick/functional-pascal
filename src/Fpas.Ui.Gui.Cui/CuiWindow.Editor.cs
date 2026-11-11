using System.Buffers;
using System.Text;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Presentation and capacity shared by native search and multiline editors.</summary>
/// <param name="Label">Optional caption above the editor.</param>
/// <param name="Placeholder">Text displayed while empty.</param>
/// <param name="Icon">Optional leading glyph.</param>
/// <param name="Hint">Supporting text below the editor.</param>
/// <param name="Error">Validation error that replaces the hint.</param>
/// <param name="Prefix">Muted text before the value.</param>
/// <param name="Suffix">Muted text after the value.</param>
/// <param name="Size">Native medium, small or large size ordinal.</param>
/// <param name="ReadOnly">Allow focus and selection while preventing edits.</param>
/// <param name="MaxUtf8Bytes">Maximum value size excluding the terminator.</param>
public sealed record CuiEditorOptions(string Label, string Placeholder, string Icon, string Hint, string Error,
    string Prefix, string Suffix, int Size, bool ReadOnly, int MaxUtf8Bytes);

public sealed partial class CuiWindow
{
    private enum EditorKind { TextField, SearchField, TextArea }

    private unsafe CuiTextFieldResult DrawEditor(EditorKind kind, string id, string value, CuiEditorOptions options,
        float height, bool password, bool clearable, bool enabled, bool focus, bool track,
        string clearControl, string? revealControl)
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
        if (!float.IsFinite(height)) throw new ArgumentOutOfRangeException(nameof(height));
        if (options.MaxUtf8Bytes <= 0 || options.MaxUtf8Bytes >= Array.MaxLength
            || (password && options.MaxUtf8Bytes > 341))
            throw new ArgumentOutOfRangeException(nameof(options.MaxUtf8Bytes));
        bool invalidLines = value.Contains('\r') || (kind != EditorKind.TextArea && value.Contains('\n'));
        if (invalidLines || Utf8.Encoding.GetByteCount(value) > options.MaxUtf8Bytes)
            throw new ArgumentException("The value must fit the UTF-8 capacity and use the editor's line endings.", nameof(value));

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
                flags = InvokeEditor(kind, stable, bytes, (nuint)capacity, options, height,
                    password ? 1 : 0, clearable ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0, nodes, out revealed);
            if (track)
            {
                TrackControl(id, nodes[0]);
                if (nodes[1] >= 0) TrackControl(clearControl, nodes[1]);
                if (nodes[2] >= 0 && revealControl is not null) TrackControl(revealControl, nodes[2]);
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

    private unsafe int InvokeEditor(EditorKind kind, string id, byte* value, nuint capacity,
        CuiEditorOptions options, float height, int password, int clearable, int enabled, int focus,
        int* nodes, out int revealed)
    {
        string? label = Optional(options.Label), placeholder = Optional(options.Placeholder);
        string? icon = Optional(options.Icon), hint = Optional(options.Hint), error = Optional(options.Error);
        string? prefix = Optional(options.Prefix), suffix = Optional(options.Suffix);
        int readOnly = options.ReadOnly ? 1 : 0;
        revealed = 0;
        return kind switch
        {
            EditorKind.SearchField => NativeMethods.SearchField(context, id, value, capacity, label, placeholder,
                icon, hint, error, prefix, suffix, options.Size, readOnly, enabled, focus, nodes),
            EditorKind.TextArea => NativeMethods.TextArea(context, id, value, capacity, label, placeholder,
                icon, hint, error, prefix, suffix, options.Size, height, clearable, readOnly, enabled, focus, nodes),
            _ => NativeMethods.TextField(context, id, value, capacity, label, placeholder,
                icon, hint, error, prefix, suffix, options.Size, password, clearable, readOnly, enabled, focus, nodes, out revealed)
        };
    }

    private static string? Optional(string value) => value.Length == 0 ? null : value;
}
