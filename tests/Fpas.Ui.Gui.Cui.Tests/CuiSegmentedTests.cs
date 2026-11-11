namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks native segments shared by public selectors and appearance controls.</summary>
public sealed class CuiSegmentedTests
{
    /// <summary>Literal hash captions retain native layout/focus, and absent glyphs reserve no icon space.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void LiteralCaptionsAndEmptyIconsKeepTheirNativeIdentity(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Literal segments", 720, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        Draw(["Day", "Week"], null);
        float textWidth = window.GetControlRectangle("group").Width;
        Draw(["Day", "Week"], ["", ""]);
        Assert.Equal(textWidth, window.GetControlRectangle("group").Width);
        Draw(["##Literal caption with enough width", "Week"], ["", ""], focus: 1);
        Assert.True(window.GetControlRectangle("first").Width > 200);
        Assert.True(window.ControlFocused("first"));
        Draw(["Renamed ### literally", "Week"], null);
        Assert.True(window.ControlFocused("first"));

        void Draw(string[] labels, string[]? icons, int focus = 0)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn();
            Assert.Equal(-1, window.Segmented("literal", labels, icons, 0, 0, false, true, focus,
                ["group", "first", "second"]));
            window.EndColumn();
            window.Present();
        }
    }

    /// <summary>Uses native text/fill sizing, removes vanished focus stops and preserves the palette.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void TextSegmentsUseNativeLayoutAndRetireRemovedChoices(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Native segments", 480, 320, renderer, continuous: true);
        window.SetReducedMotion(true);
        uint[] palette = window.ReadTheme();
        Assert.Equal(-1, Draw(["Day", "Week", "Month"], 0, focus: 3));
        Assert.Equal(432, window.GetControlRectangle("group").Width);
        Assert.True(window.ControlFocused("item/2"));
        Assert.Equal(2, Draw(["Day", "Week", "Month"], 0, key: 2));
        Assert.Equal(-1, Draw(["Day"], 0));
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("item/2"));
        Assert.False(window.ControlFocused("item/0"));
        Assert.Equal(-1, Draw(["Day", "Week"], 0, focus: 2));
        Assert.Equal(-1, Draw(["Day", "Week"], 0, enabled: false, key: 2));
        Assert.False(window.ControlFocused("item/1"));
        Assert.Equal(palette, window.ReadTheme());

        int Draw(string[] labels, int selected, int focus = 0, int key = 0, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn();
            string[] tracking = ["group", .. Enumerable.Range(0, labels.Length).Select(index => "item/" + index)];
            int proposal = window.Segmented("segments", labels, null, selected, 0, true, enabled, focus, tracking);
            window.EndColumn();
            window.Present();
            return proposal;
        }
    }
}
