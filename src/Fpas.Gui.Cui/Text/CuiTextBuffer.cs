using System.Globalization;
using Fpas.Gui.Cui.Interop;

namespace Fpas.Gui.Cui.Text;

/// <summary>Owns bounded, zero-terminated UTF-8 text for a native editor. Synchronize access with its UI thread.</summary>
public sealed class CuiTextBuffer
{
    internal byte[] Bytes { get; }

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

    /// <summary>Replaces the text without truncation; invalid input leaves it unchanged.</summary>
    /// <param name="text">Valid Unicode text that fits with its terminating zero.</param>
    public void SetText(string text)
    {
        Utf8.Validate(text, nameof(text));
        int length = Utf8.Encoding.GetByteCount(text);
        if (length >= Bytes.Length) throw new ArgumentException("Text does not fit the UTF-8 buffer.", nameof(text));
        Bytes.AsSpan().Clear();
        Utf8.Encoding.GetBytes(text, Bytes);
    }

    /// <summary>Converts a UTF-16 grapheme boundary into its UTF-8 byte offset.</summary>
    /// <param name="utf16Offset">A grapheme start or the document end.</param>
    /// <returns>The corresponding byte offset.</returns>
    public int GetUtf8Offset(int utf16Offset)
    {
        string text = Text;
        CheckBoundary(text, utf16Offset, nameof(utf16Offset));
        return Utf8.Encoding.GetByteCount(text.AsSpan(0, utf16Offset));
    }

    /// <summary>Converts a UTF-8 grapheme boundary into its UTF-16 offset.</summary>
    /// <param name="utf8Offset">A byte offset at a grapheme start or document end.</param>
    /// <returns>The corresponding UTF-16 offset.</returns>
    public int GetUtf16Offset(int utf8Offset)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(utf8Offset);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(utf8Offset, Length);
        int offset = Utf8.Encoding.GetCharCount(Bytes.AsSpan(0, utf8Offset));
        CheckBoundary(Text, offset, nameof(utf8Offset));
        return offset;
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

    private static void CheckBoundary(string text, int offset, string parameter)
    {
        if (offset < 0 || offset > text.Length) throw new ArgumentOutOfRangeException(parameter);
        if (offset != text.Length && Array.BinarySearch(StringInfo.ParseCombiningCharacters(text), offset) < 0)
            throw new ArgumentException("Position splits a Unicode grapheme.", parameter);
    }
}
