using Fpas.Ui.Gui.Cui.Text;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises document and selection updates through the native text area.</summary>
public sealed class CuiTextEditingTests
{
    /// <summary>Places the next edit after the entire grapheme when typing creates a combining sequence.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void NativeCaretReturnsToAGraphemeBoundaryAfterTyping()
    {
        using var window = new CuiWindow("Grapheme editing", renderer: CuiRenderer.Software, continuous: true);
        var text = new CuiTextBuffer("\u0301\nnext");
        const string id = "editor";

        Assert.True(window.NextFrame());
        window.RequestFocus(id);
        window.TextArea(id, text);

        Assert.True(window.NextFrame());
        window.TypeText("e");
        window.TextArea(id, text);
        Assert.Equal(new CuiTextSelection(2, 2), text.Selection);

        Assert.True(window.NextFrame());
        window.TypeText("!");
        window.TextArea(id, text);
        Assert.Equal("e\u0301!\nnext", text.Text);
        Assert.Equal(new CuiTextSelection(3, 3), text.Selection);
    }

    /// <summary>Uses the host's directional selection and edits the replacement at that position.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public void TextAreaAppliesReplacementAndDirectionalSelection()
    {
        using var window = new CuiWindow("Document replacement", renderer: CuiRenderer.Software, continuous: true);
        var text = new CuiTextBuffer("earlier document");
        const string id = "editor";
        const string prefix = "Grüße\n";
        const string cluster = "🧑‍💻";

        Assert.True(window.NextFrame());
        window.RequestFocus(id);
        window.TextArea(id, text);

        Assert.True(window.NextFrame());
        var selection = new CuiTextSelection(prefix.Length + cluster.Length, prefix.Length);
        text.SetText(prefix + cluster + " tail", selection);
        var events = window.TextArea(id, text);
        Assert.False(events.HasFlag(CuiTextInputEvents.Changed));
        Assert.Equal(selection, text.Selection);

        Assert.True(window.NextFrame());
        window.TypeText("新");
        window.TextArea(id, text);
        Assert.Equal(prefix + "新 tail", text.Text);
        Assert.Equal(new CuiTextSelection(prefix.Length + 1, prefix.Length + 1), text.Selection);
    }
}
