using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>Code editing and caret notifications.</summary>
/// <param name="Text">Edited text when it changed.</param>
/// <param name="Line">One-based caret line.</param>
/// <param name="Column">One-based caret column.</param>
/// <param name="Lines">Number of lines.</param>
/// <param name="Refused">Whether the requested edit exceeded capacity.</param>
public sealed record CuiCodeEditorEvent(string? Text, int Line, int Column, int Lines, bool Refused);

public sealed partial class CuiWindow
{
    /// <summary>Draws the native CodeEditor widget.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal source text.</param>
    /// <param name="capacity">Maximum UTF-8 buffer size including the terminator.</param>
    /// <param name="language">Native syntax language.</param>
    /// <param name="placeholder">Hint shown when the editor is empty.</param>
    /// <param name="widthKind">Native extent kind.</param>
    /// <param name="width">Extent value for fixed and percentage widths.</param>
    /// <param name="height">Logical height, or the native auto/fill sentinel.</param>
    /// <param name="size">Logical font size, or zero for the default.</param>
    /// <param name="tabSize">Columns in an indentation level.</param>
    /// <param name="flags">Widget-specific input and rendering flags.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public unsafe CuiCodeEditorEvent? CodeEditor(string id, string text, int capacity, string language, string placeholder, int widthKind, float width, float height, float size, int tabSize, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        Utf8.Validate(text, nameof(text));
        int length = Utf8.Encoding.GetByteCount(text);
        if (capacity <= length) throw new ArgumentOutOfRangeException(nameof(capacity));
        var buffer = System.Buffers.ArrayPool<byte>.Shared.Rent(capacity);
        try
        {
            Utf8.Encoding.GetBytes(text, buffer);
            buffer[length] = 0;
            int changes;
            int line, column, lines;
            fixed (byte* source = buffer)
                changes = NativeMethods.CodeEditor(context, Stable(id), source, (nuint)capacity, language, OptionalAi(placeholder),
                    widthKind, width, height, size, tabSize, flags, out line, out column, out lines);
            if (changes == 0) return null;
            return new((changes & 1) == 0 ? null : Utf8.Encoding.GetString(buffer, 0, Array.IndexOf(buffer, (byte)0, 0, capacity)),
                line, column, lines, (changes & 2) != 0);
        }
        finally
        {
            System.Buffers.ArrayPool<byte>.Shared.Return(buffer, clearArray: true);
        }
    }
}
