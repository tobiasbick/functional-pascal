using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Checks portable button options crossing the native CUI boundary.</summary>
public sealed class CuiButtonTests
{
    /// <summary>Joins adjacent buttons and restores standalone button styling after the group.</summary>
    [GuiFact]
    public void ButtonGroupRestoresItsNodeAndNativeScope()
    {
        using var window = new CuiWindow("Button group", 480, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.True(window.NextFrame());
        window.MovePointer(-1, -1);
        window.BeginColumn();
        window.BeginButtonGroup();
        window.Button("Bold");
        window.TrackLastControl("bold");
        window.Button("Italic");
        window.TrackLastControl("italic");
        window.EndButtonGroup();
        window.TrackLastControl("group");
        window.Button("Standalone");
        window.TrackLastControl("standalone");
        window.EndColumn();
        window.Present();

        var bold = window.GetControlRectangle("bold");
        var italic = window.GetControlRectangle("italic");
        var group = window.GetControlRectangle("group");
        Assert.Equal(bold.Y, italic.Y);
        Assert.InRange(italic.X - (bold.X + bold.Width), -0.25f, 0.25f);
        Assert.InRange(group.Width - (bold.Width + italic.Width), -0.25f, 0.25f);
        var standalone = window.GetControlRectangle("standalone");
        using var directory = new TemporaryDirectory();
        string capture = directory.File("group.png");
        window.CapturePng(capture);
        uint[] pixels = PngAssertions.ReadPixels(capture, 480, 320);
        Assert.NotEqual(Face(bold), Face(standalone));

        uint Face(CuiRectangle button) => pixels[((int)button.Y + (int)button.Height / 2) * 480 + (int)button.X + 4];
    }

    /// <summary>Keeps focus through label and icon changes and respects busy and disabled input.</summary>
    [GuiFact]
    public void ButtonKeepsIdentityAndAppliesSizeAndInputOptions()
    {
        using var window = new CuiWindow("Button", 480, 320, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        Assert.False(Draw("Save ## literal", "\uE13D", size: 1, focus: true));
        Assert.Equal(30, window.GetControlRectangle("button").Height);
        Assert.True(Draw("Save", "\uE06C", key: 2));
        Assert.Equal(36, window.GetControlRectangle("button").Height);
        Assert.True(window.ControlFocused("button"));
        Assert.False(Draw("Saving", "\uE06C", key: 6, size: 2, loading: true));
        Assert.Equal(44, window.GetControlRectangle("button").Height);
        Assert.Equal(192, window.GetControlRectangle("button").Width);
        Assert.True(window.ControlFocused("button"));
        Assert.False(Draw("Save", "\uE06C", key: 2, enabled: false));
        Assert.False(window.ControlFocused("button"));

        bool Draw(string label, string icon, int key = 0, int size = 0,
            bool enabled = true, bool loading = false, bool focus = false)
        {
            Assert.True(window.NextFrame());
            if (key != 0)
            {
                window.SetKey(key, true);
                window.SetKey(key, false);
            }
            window.BeginColumn();
            bool activated = window.Button("button", label, 1, size, enabled, loading, true, focus,
                icon, "\uE049", "Save changes", "Ctrl S", false, 192, 0xff649628);
            window.TrackLastControl("button");
            window.EndColumn();
            window.Present();
            return activated;
        }
    }
}
