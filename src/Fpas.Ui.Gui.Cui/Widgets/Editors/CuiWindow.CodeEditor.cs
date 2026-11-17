using Fpas.Ui.Gui.Cui.Widgets.Editors;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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
    /// <param name="marks">Optional highlights sorted by UTF-16 start offset.</param>
    /// <returns>Input proposals or an updated native observation.</returns>
    public unsafe CuiCodeEditorEvent? CodeEditor(string id, string text, int capacity, string language, string placeholder, int widthKind, float width, float height, float size, int tabSize, int flags, CuiTextMark[]? marks = null)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(language, nameof(language));
        Utf8.Validate(placeholder, nameof(placeholder));
        if (tabSize is < 1 or > 16) throw new ArgumentOutOfRangeException(nameof(tabSize));
        var nativeMarks = new NativeTextMark[marks?.Length ?? 0];
        int previous = -1;
        for (int index = 0; index < nativeMarks.Length; index++)
        {
            var mark = marks![index];
            if (mark.Start < previous || mark.End < mark.Start || mark.Color is < 0 or > 6)
                throw new ArgumentException("Highlights must be sorted ranges with native colors zero through six.", nameof(marks));
            nativeMarks[index] = new() { Start = TextOffset(text, mark.Start), End = TextOffset(text, mark.End), Color = mark.Color };
            previous = mark.Start;
        }
        int length = Utf8.Encoding.GetByteCount(text);
        if (capacity <= length) throw new ArgumentOutOfRangeException(nameof(capacity));
        var buffer = System.Buffers.ArrayPool<byte>.Shared.Rent(capacity);
        try
        {
            Utf8.Encoding.GetBytes(text, buffer);
            buffer[length] = 0;
            int changes;
            NativeCodeEditor result;
            flags = ApplyEditorRequest(id, text, flags);
            fixed (byte* source = buffer)
            fixed (NativeTextMark* highlights = nativeMarks)
                changes = NativeMethods.CodeEditor(context, Stable(id), source, (nuint)capacity, language, OptionalAi(placeholder),
                    widthKind, width, height, size, tabSize, flags, highlights, nativeMarks.Length, out result);
            if (changes == 0) return null;
            return new((changes & 1) == 0 ? null : Utf8.Encoding.GetString(buffer, 0, Array.IndexOf(buffer, (byte)0, 0, capacity)),
                result.Line, result.Column, result.Lines, (changes & 2) != 0,
                Utf8.Encoding.GetCharCount(buffer, 0, result.Cursor), Utf8.Encoding.GetCharCount(buffer, 0, result.Anchor),
                result.Matches, result.MatchIndex, result.Focused != 0);
        }
        finally
        {
            System.Buffers.ArrayPool<byte>.Shared.Return(buffer, clearArray: true);
        }
    }
}
