using System.Text;
using Fpas.Gui.Cui.Text;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Verifies bounded encoding and document-position contracts without a display.</summary>
public sealed class CuiTextBufferTests
{
    /// <summary>Counts UTF-8 bytes, reserves the terminator and preserves text on invalid writes.</summary>
    [Fact]
    public void CapacityAndInvalidWritesPreserveTheBuffer()
    {
        var buffer = new CuiTextBuffer("ä", 3);
        Assert.Equal(3, buffer.Capacity);
        Assert.Throws<ArgumentException>(() => buffer.SetText("😀"));
        Assert.Equal("ä", buffer.Text);
        Assert.Throws<ArgumentException>(() => buffer.SetText("a\0b"));
        Assert.Equal("ä", buffer.Text);
        Assert.Throws<EncoderFallbackException>(() => buffer.SetText("\ud800"));
        Assert.Equal("ä", buffer.Text);
        buffer.SetText("");
        Assert.Equal("", buffer.Text);
        Assert.Throws<ArgumentOutOfRangeException>(() => new CuiTextBuffer(capacity: 1));
    }

    /// <summary>Round-trips valid grapheme boundaries through UTF-8 and UTF-16.</summary>
    /// <param name="utf16">The expected document offset.</param>
    /// <param name="utf8">The corresponding byte offset.</param>
    [Theory]
    [InlineData(0, 0)]
    [InlineData(1, 1)]
    [InlineData(2, 3)]
    [InlineData(4, 6)]
    [InlineData(6, 10)]
    [InlineData(7, 11)]
    public void UnicodeOffsetsRoundTrip(int utf16, int utf8)
    {
        var buffer = new CuiTextBuffer("Aäe\u0301😀Z");
        Assert.Equal(utf8, buffer.GetUtf8Offset(utf16));
        Assert.Equal(utf16, buffer.GetUtf16Offset(utf8));
    }

    /// <summary>Rejects positions inside combining sequences, encoded scalars and surrogate pairs.</summary>
    [Fact]
    public void InvalidDocumentBoundariesAreRejected()
    {
        var buffer = new CuiTextBuffer("Aäe\u0301😀Z");
        Assert.Throws<ArgumentException>(() => buffer.GetUtf8Offset(3));
        Assert.Throws<ArgumentException>(() => buffer.GetUtf8Offset(5));
        Assert.Throws<ArgumentException>(() => buffer.GetUtf16Offset(4));
        Assert.Throws<DecoderFallbackException>(() => buffer.GetUtf16Offset(2));
        Assert.Throws<DecoderFallbackException>(() => buffer.GetUtf16Offset(7));
        Assert.Throws<ArgumentOutOfRangeException>(() => buffer.GetUtf16Offset(12));
        Assert.Throws<ArgumentOutOfRangeException>(() => buffer.GetUtf8Offset(-1));
        var empty = new CuiTextBuffer();
        Assert.Equal(0, empty.GetUtf8Offset(0));
        Assert.Equal(0, empty.GetUtf16Offset(0));
    }

    /// <summary>Preserves the current document and selection when replacement positions split a grapheme.</summary>
    [Fact]
    public void InvalidReplacementPreservesDocumentAndSelection()
    {
        var buffer = new CuiTextBuffer("a😀");
        var selection = new CuiTextSelection(3, 1);
        buffer.Selection = selection;

        Assert.Throws<ArgumentException>(() => buffer.SetText("e\u0301", new(1, 0)));

        Assert.Equal("a😀", buffer.Text);
        Assert.Equal(selection, buffer.Selection);
    }
}
