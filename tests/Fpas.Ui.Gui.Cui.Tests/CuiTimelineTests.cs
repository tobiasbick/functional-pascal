using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks native activity geometry, literal text and the managed ABI.</summary>
public sealed class CuiTimelineTests
{
    /// <summary>Wrapping follows the offered width, and the last flag removes native trailing space.</summary>
    /// <param name="renderer">Native renderer to exercise.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void NativeItemsWrapResizeAndOmitTheFinalConnector(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Activity geometry", 640, 400, renderer, continuous: true);
        float wideHeight = 0;
        foreach (int width in new[] { 640, 280, 640, 280 })
        {
            window.Resize(width, 400);
            Draw(false); Draw(false);
            var regular = window.GetControlRectangle("entry");
            Assert.Equal(width - 24, regular.Width);
            Assert.False(window.ControlFocused("entry"));
            if (width == 640) wideHeight = regular.Height;
            else Assert.True(regular.Height > wideHeight);
            Draw(true); Draw(true);
            Assert.True(window.GetControlRectangle("entry").Height < regular.Height);
        }

        void Draw(bool last)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn(12);
            window.TimelineItem("entry", "Deployment complete", "2m ago",
                "The release is live in every region; supporting text wraps using native CUI layout.",
                tone: 2, last: last);
            window.TrackLastControl("entry");
            window.EndColumn();
            window.Present();
        }
    }

    /// <summary>Title, time and description independently preserve literal CUI identity delimiters.</summary>
    /// <param name="renderer">Native renderer to exercise.</param>
    /// <param name="field">Title zero, time one, or description two.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, 0)]
    [InlineData(CuiRenderer.Software, 1)]
    [InlineData(CuiRenderer.Software, 2)]
    [InlineData(CuiRenderer.OpenGl, 0)]
    [InlineData(CuiRenderer.OpenGl, 1)]
    [InlineData(CuiRenderer.OpenGl, 2)]
    public void EachTextFieldIsLiteral(CuiRenderer renderer, int field)
    {
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("Literal activity", 500, 240, renderer, continuous: true);
        string full = directory.File("full.png"), shortText = directory.File("short.png");
        Draw(true); Draw(true); window.CapturePng(full);
        Draw(false); Draw(false); window.CapturePng(shortText);
        Assert.False(PngAssertions.ReadPixels(full, 500, 240)
            .SequenceEqual(PngAssertions.ReadPixels(shortText, 500, 240)));

        void Draw(bool literal)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            string suffix = literal ? "##literal λ" : "";
            window.TimelineItem("entry", "Deployment" + (field == 0 ? suffix : ""),
                "2m" + (field == 1 ? suffix : ""), "Release" + (field == 2 ? suffix : ""));
            window.TrackLastControl("entry");
            window.Present();
        }
    }

    /// <summary>Rejected text and tones leave the native frame usable.</summary>
    [GuiFact]
    public void InvalidOptionsDoNotCorruptTheFrame()
    {
        using var window = new CuiWindow("Activity validation", renderer: CuiRenderer.Software, continuous: true);
        Assert.True(window.NextFrame());
        Assert.Throws<ArgumentException>(() => window.TimelineItem("bad", " "));
        Assert.Throws<ArgumentException>(() => window.TimelineItem("bad", "zero\0"));
        Assert.Throws<ArgumentException>(() => window.TimelineItem("bad", "Title", time: "zero\0"));
        Assert.Throws<ArgumentException>(() => window.TimelineItem("bad", "Title", description: "zero\0"));
        Assert.Throws<ArgumentException>(() => window.TimelineItem("bad", "Title", icon: "zero\0"));
        foreach (int tone in new[] { -1, 9 })
            Assert.Throws<ArgumentOutOfRangeException>(() => window.TimelineItem("bad", "Title", tone: tone));
        window.TimelineItem("valid", "Title");
        window.Present();
    }
}
