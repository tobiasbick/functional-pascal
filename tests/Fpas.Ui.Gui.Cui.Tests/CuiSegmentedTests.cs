namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks the native segment primitive shared by appearance controls.</summary>
public sealed class CuiSegmentedTests
{
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
