namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises the native disclosure widgets through their managed bridge.</summary>
public sealed class CuiDisclosureTests
{
    /// <summary>Headers keep literal captions, model proposals and native pointer/keyboard behavior.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software, false)]
    [InlineData(CuiRenderer.OpenGl, false)]
    [InlineData(CuiRenderer.Software, true)]
    [InlineData(CuiRenderer.OpenGl, true)]
    public void HeadersProposeWithoutOwningTheModel(CuiRenderer renderer, bool card)
    {
        using var window = new CuiWindow("Native disclosure", 600, 480, renderer, continuous: true);
        window.SetReducedMotion(true);
        Assert.Equal(0, Draw(focus: true));
        Assert.True(window.ControlFocused("header"));
        Assert.True(window.GetControlRectangle("header").Width > 500);
        Assert.Equal(7, Draw(key: 2));
        Assert.Equal(0, Draw()); // The model declines the opening proposal.
        Assert.True(window.ControlFocused("header"));
        Assert.Equal(7, Draw(key: 6, title: "Renamed ### literally"));
        Assert.Equal(5, Draw(open: true));
        Assert.Equal(2, Draw(open: true, key: 2));
        Assert.Equal(5, Draw(open: true)); // The model declines closing.
        Assert.Equal(5, Draw(open: true, key: 3));
        Assert.Equal(0, Draw(enabled: false, key: 6));
        Assert.False(window.ControlFocused("header"));

        Draw();
        var header = window.GetControlRectangle("header");
        Draw(pointer: header);
        Draw(pointer: header, mouse: true);
        Assert.Equal(7, Draw(pointer: header, mouse: false));
        Draw(open: true);
        var body = window.GetControlRectangle("body");
        Assert.True(body.Y >= header.Y + header.Height);
        window.ForgetControl("header");
        Assert.Equal(0, Draw());

        int Draw(bool open = false, bool enabled = true, bool focus = false, int key = 0,
            string title = "Title ## literal – λ", CuiRectangle? pointer = null, bool? mouse = null)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(pointer is { } p ? p.X + 8 : -1, pointer is { } q ? q.Y + 8 : -1);
            if (mouse is { } down) window.SetLeftButton(down);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn(12, 8);
            int result = window.BeginDisclosure("header", title, card, open, enabled, focus,
                card ? "Supporting text" : "", badge: card ? "Active" : "", tone: 2);
            if ((result & 1) != 0)
            {
                window.Button("Body action");
                window.TrackLastControl("body");
                window.EndDisclosure();
            }
            window.EndColumn();
            window.Present();
            return result;
        }
    }

    /// <summary>Native clamping responds to width and retires local expansion when removed.</summary>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void ExpandableTextUsesNativeLocalStateAndOptionalLink(CuiRenderer renderer)
    {
        using var window = new CuiWindow("Expandable text", 500, 500, renderer, continuous: true);
        window.SetReducedMotion(true);
        string longText = string.Concat(Enumerable.Repeat("Literal ## λ text wraps to several lines. ", 12));
        Draw(longText);
        Assert.Equal(4, Draw(longText, focus: true));
        Assert.True(window.ControlFocused("toggle"));
        float clamped = window.GetControlRectangle("text").Height;
        Assert.Equal(7, Draw(longText, key: 2));
        Assert.Equal(6, Draw(longText));
        Assert.True(window.GetControlRectangle("text").Height > clamped);
        Assert.Equal(6, Draw(longText, enabled: false, key: 6));
        Assert.Equal(5, Draw(longText, focus: true, key: 6));
        Assert.Equal(4, Draw(longText));
        window.Resize(280, 500);
        Draw(longText);
        Draw(longText);
        Assert.Equal(4, Draw(longText));
        window.Resize(700, 500);
        Draw("Short text");
        Assert.Equal(0, Draw("Short text"));
        Assert.Throws<KeyNotFoundException>(() => window.GetControlRectangle("toggle"));
        Draw(longText);
        Assert.Equal(7, Draw(longText, focus: true, key: 2));
        Assert.True(window.NextFrame());
        window.Text("Text removed");
        window.Present();
        Draw(longText);
        Assert.Equal(4, Draw(longText));

        int Draw(string text, bool focus = false, int key = 0, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            if (key != 0) { window.SetKey(key, true); window.SetKey(key, false); }
            window.BeginColumn(12, 8);
            int result = window.ExpandableText("text", text, 2, 0, 0, 0, enabled, focus, "toggle");
            window.EndColumn();
            window.Present();
            return result;
        }
    }

    /// <summary>Mixed nested native bodies close cleanly and reject capacity overflow before mutation.</summary>
    [GuiFact]
    public void NestedDisclosuresBalanceInputAndLayoutScopes()
    {
        using var window = new CuiWindow("Nested disclosures", renderer: CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.True(window.NextFrame());
        window.BeginColumn();
        int opened = 0;
        try
        {
            for (; opened < 40; opened++)
                window.BeginDisclosure("level/" + opened, "Nested", opened % 2 == 0, true, true, false);
            Assert.Fail("Native stack overflow was accepted.");
        }
        catch (InvalidOperationException)
        {
            Assert.InRange(opened, 15, 32);
        }
        for (int index = 0; index < opened; index++) window.EndDisclosure();
        Assert.False(window.Button("Input still works"));
        window.EndColumn();
        window.Present();
        Assert.True(window.NextFrame());
        window.SetKey(2, true);
        window.SetKey(2, false);
        window.BeginColumn();
        Assert.Equal(7, window.BeginDisclosure("fresh", "Fresh", false, false, true, true));
        window.EndDisclosure();
        window.EndColumn();
    }
}
