using System.Globalization;
using Fpas.Gui.Cui.Interop;

namespace Fpas.Gui.Cui.Text;

/// <summary>Owns bounded, zero-terminated UTF-8 text for a native editor. Synchronize access with its UI thread.</summary>
public sealed class CuiTextBuffer
{
    private static long nextDocumentVersion;
    private CuiTextSelection selection;

    internal byte[] Bytes { get; }
    internal long DocumentVersion { get; private set; }
    internal (int Anchor, int Caret) ByteSelection { get; private set; }

    /// <summary>Creates an editing buffer.</summary>
    /// <param name="text">Valid initial Unicode text without embedded zeros.</param>
    /// <param name="capacity">Byte capacity including the terminating zero.</param>
    public CuiTextBuffer(string text = "", int capacity = 4096)
    {
        ArgumentOutOfRangeException.ThrowIfLessThan(capacity, 2);
        Bytes = new byte[capacity];
        SetText(text);
    }

    /// <summary>Gets the current decoded text.</summary>
    public string Text => Utf8.Encoding.GetString(Bytes, 0, Length);

    /// <summary>Gets the byte capacity including the terminating zero.</summary>
    public int Capacity => Bytes.Length;

    /// <summary>Gets or sets the directional selection at UTF-16 grapheme boundaries.</summary>
    public CuiTextSelection Selection
    {
        get => selection;
        set
        {
            ByteSelection = EncodeSelection(Text, value, nameof(value));
            selection = value;
        }
    }

    /// <summary>Replaces the document and places the caret at the start.</summary>
    /// <param name="text">Valid Unicode text that fits with its terminating zero.</param>
    public void SetText(string text) => SetText(text, default);

    /// <summary>Replaces the document and selection together. Invalid input leaves both unchanged.</summary>
    /// <param name="text">Valid Unicode text that fits with its terminating zero.</param>
    /// <param name="selection">Directional selection at UTF-16 grapheme boundaries in the new document.</param>
    public void SetText(string text, CuiTextSelection selection)
    {
        Utf8.Validate(text, nameof(text));
        int length = Utf8.Encoding.GetByteCount(text);
        if (length >= Bytes.Length) throw new ArgumentException("Text does not fit the UTF-8 buffer.", nameof(text));
        var byteSelection = EncodeSelection(text, selection, nameof(selection));
        Bytes.AsSpan().Clear();
        Utf8.Encoding.GetBytes(text, Bytes);
        this.selection = selection;
        ByteSelection = byteSelection;
        DocumentVersion = Interlocked.Increment(ref nextDocumentVersion);
    }

    /// <summary>Converts a UTF-16 grapheme boundary into its UTF-8 byte offset.</summary>
    /// <param name="utf16Offset">A grapheme start or the document end.</param>
    /// <returns>The corresponding byte offset.</returns>
    public int GetUtf8Offset(int utf16Offset)
    {
        string text = Text;
        return EncodeOffset(text, StringInfo.ParseCombiningCharacters(text), utf16Offset, nameof(utf16Offset));
    }

    /// <summary>Converts a UTF-8 grapheme boundary into its UTF-16 offset.</summary>
    /// <param name="utf8Offset">A byte offset at a grapheme start or document end.</param>
    /// <returns>The corresponding UTF-16 offset.</returns>
    public int GetUtf16Offset(int utf8Offset)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(utf8Offset);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(utf8Offset, Length);
        int offset = Utf8.Encoding.GetCharCount(Bytes.AsSpan(0, utf8Offset));
        string text = Text;
        CheckBoundary(text.Length, StringInfo.ParseCombiningCharacters(text), offset, nameof(utf8Offset));
        return offset;
    }

    internal void ReadNativeSelection(int anchor, int caret, bool textChanged)
    {
        if (!textChanged && ByteSelection == (anchor, caret)) return;
        string text = Text;
        int[] boundaries = StringInfo.ParseCombiningCharacters(text);
        int utf16Caret = SnapToBoundary(text.Length, boundaries, caret, ByteSelection.Caret);
        int utf16Anchor = anchor == caret
            ? utf16Caret
            : SnapToBoundary(text.Length, boundaries, anchor, ByteSelection.Anchor);
        selection = new(utf16Anchor, utf16Caret);
        ByteSelection = (Utf8.Encoding.GetByteCount(text.AsSpan(0, utf16Anchor)),
            Utf8.Encoding.GetByteCount(text.AsSpan(0, utf16Caret)));
    }

    private int Length
    {
        get
        {
            int length = Array.IndexOf(Bytes, (byte)0);
            if (length < 0) throw new InvalidOperationException("Native editing removed the buffer terminator.");
            return length;
        }
    }

    private int SnapToBoundary(int length, int[] boundaries, int position, int previous)
    {
        int offset = Utf8.Encoding.GetCharCount(Bytes.AsSpan(0, position));
        if (offset == length) return offset;
        int index = Array.BinarySearch(boundaries, offset);
        if (index >= 0) return offset;
        // Continue in the direction of motion so repeated arrows can cross a multi-scalar grapheme.
        int next = ~index;
        if (position <= previous) return boundaries[next - 1];
        return next == boundaries.Length ? length : boundaries[next];
    }

    private static (int Anchor, int Caret) EncodeSelection(string text, CuiTextSelection selection, string parameter)
    {
        int[] boundaries = StringInfo.ParseCombiningCharacters(text);
        return (EncodeOffset(text, boundaries, selection.Anchor, parameter),
            EncodeOffset(text, boundaries, selection.Caret, parameter));
    }

    private static int EncodeOffset(string text, int[] boundaries, int offset, string parameter)
    {
        CheckBoundary(text.Length, boundaries, offset, parameter);
        return Utf8.Encoding.GetByteCount(text.AsSpan(0, offset));
    }

    private static void CheckBoundary(int length, int[] boundaries, int offset, string parameter)
    {
        if (offset < 0 || offset > length) throw new ArgumentOutOfRangeException(parameter);
        if (offset != length && Array.BinarySearch(boundaries, offset) < 0)
            throw new ArgumentException("Position splits a Unicode grapheme.", parameter);
    }
}
