// [Documentation](../../../docs/ui/gui/more-developer-tools/http-request-bar.md#query-parameters)
using Fpas.Ui.Gui.Cui.Widgets.Developer;

namespace Fpas.Ui.Gui.Cui.Tests.Developer;

/// <summary>Exercises CUI query editing without creating a window.</summary>
public sealed class CuiUrlParametersTests
{
    /// <summary>Decodes query values and preserves row order up to the capacity.</summary>
    [Fact]
    public void ParseDecodesPairsBeforeTheFragment()
    {
        CuiDevPair[] expected = [new("q", "two words", true, false), new("q", "é&", true, false), new("draft", "", true, false)];
        const string url = "/items?q=two+words&q=%C3%A9%26&draft#notes?disabled=1";
        Assert.Equal(expected, CuiWindow.UrlParameters(url, 8));
        Assert.Equal(expected[..2], CuiWindow.UrlParameters(url, 2));
        Assert.Empty(CuiWindow.UrlParameters(url, 0));
        Assert.Empty(CuiWindow.UrlParameters("/items#notes?draft=1", 8));
    }

    /// <summary>Ignores the trailing byte of a scalar that does not fit in the native key.</summary>
    [Fact]
    public void ParseKeepsCompleteUnicodeAtTheNativeBoundary()
    {
        string prefix = new('x', 62);
        CuiDevPair row = Assert.Single(CuiWindow.UrlParameters("/items?" + prefix + "%C3%A9=ok", 1));
        Assert.Equal(new CuiDevPair(prefix, "ok", true, false), row);
    }

    /// <summary>Treats the native CUI buffer limit as an exact UTF-8 byte count.</summary>
    [Fact]
    public void ReplacePreservesTheFragmentAndAcceptsAnExactFit()
    {
        string fragment = "#notes?old=1" + new string('f', 300) + "é";
        CuiDevPair[] rows = [new("q", "a&b", true, true), new("", "ignored", true, false), new("skip", "x", false, false), new("a", "", true, false)];
        string expected = "/items?q=a%26b&a" + fragment;
        int bytes = System.Text.Encoding.UTF8.GetByteCount(expected);
        Assert.Equal(expected, CuiWindow.SetUrlParameters("/items" + fragment, rows, bytes));
        Assert.Null(CuiWindow.SetUrlParameters("/items" + fragment, rows, bytes - 1));
        Assert.Equal("/x?a", CuiWindow.SetUrlParameters("/x", [new("a", "", true, false)], 4));
        Assert.Null(CuiWindow.SetUrlParameters("/x", [new("a", "", true, false)], 3));
    }

    /// <summary>Removes the old query when no enabled pair has a key.</summary>
    [Fact]
    public void ReplaceCanRemoveAQueryLargerThanTheResultBuffer()
    {
        Assert.Equal("/x#note", CuiWindow.SetUrlParameters("/x?old=value#note", [], 7));
    }
}
