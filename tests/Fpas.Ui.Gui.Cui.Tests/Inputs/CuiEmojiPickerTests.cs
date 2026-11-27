// [Documentation](../../../docs/ui/gui/smart-inputs/emoji-picker.md)
using Fpas.Ui.Gui.Cui.Widgets.Pickers;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks native emoji picker options and history capacity.</summary>
public sealed class CuiEmojiPickerTests
{
    /// <summary>Supports native-owned state, custom group captions and percentage width.</summary>
    [GuiFact]
    public void PickerAcceptsNativeStateAndLayoutOptions()
    {
        using var window = new CuiWindow("Emoji options", 600, 400, CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        string[] names = Enumerable.Range(0, 10).Select(index => $"Group {index}").ToArray();
        var result = window.EmojiPicker("picker", null, 0, null, 3, 50, 160, 22,
            "", "", "", names, false, false, false, true, 0, part => $"part-{part}");
        window.Present();

        Assert.Null(result.Recent);
        Assert.Null(result.Tone);
        Assert.Null(result.Emoji);
        Assert.Equal(window.Size.Width / 2, window.GetControlRectangle("picker").Width);
        Assert.Equal(160, window.GetControlRectangle("part-1").Height);
        Assert.DoesNotContain(0, result.VisibleParts);
        Assert.DoesNotContain(2, result.VisibleParts);
    }

    /// <summary>Keeps prior history when a complete selection cannot fit in its buffer.</summary>
    [GuiFact]
    public void ShortHistoryStillReturnsCompleteEmoji()
    {
        const string prior = "😀", rainbowFlag = "🏳️‍🌈";
        using var window = new CuiWindow("Emoji history capacity", 600, 450, CuiRenderer.Software, continuous: true);
        if (!window.LoadSystemEmojiFont() || !window.SupportsEmoji(rainbowFlag)) return;
        window.SetReducedMotion(true);
        Draw(focus: 1);
        Draw(query: "rainbow flag");
        Draw();
        var result = Draw(enter: true);
        Assert.Equal(rainbowFlag, result.Emoji);
        Assert.Equal(prior, result.Recent);

        CuiEmojiPickerResult Draw(int focus = 0, string? query = null, bool enter = false)
        {
            Assert.True(window.NextFrame());
            if (query is not null) window.InputText(query);
            if (enter) { window.SetKey(2, true); window.SetKey(2, false); }
            var result = window.EmojiPicker("picker", prior, 10, 0, 0, 0, 160, 0,
                null, null, null, null, true, true, true, true, focus, null);
            window.Present();
            return result;
        }
    }
}
