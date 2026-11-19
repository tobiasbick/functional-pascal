// [Documentation](../../../docs/ui/gui/disclosure/avatar.md)
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Widgets.Canvas;

namespace Fpas.Ui.Gui.Cui.Tests.Media;

/// <summary>Exercises avatar images and initials in CUI widgets.</summary>
public sealed class CuiAvatarTests
{
    /// <summary>A model-owned path replaces the native initials and clearing it restores them.</summary>
    [GuiFact]
    public void AvatarReplacesAndClearsAnImage()
    {
        using var directory = new TemporaryDirectory();
        string path = directory.File("画像.png");
        File.WriteAllBytes(path, TestImages.Blue);
        const int width = 400, height = 320;
        using var window = new CuiWindow("CUI avatar image", width, height, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);

        Assert.False(Draw(null));
        var bounds = window.GetControlRectangle("avatar");
        int center = (int)(bounds.Y + bounds.Height / 2) * width + (int)(bounds.X + bounds.Width / 2);
        uint initials = Sample("initials");
        Assert.True(Draw(path));
        Assert.Equal(0xffdc641eu, Sample("image"));
        Assert.False(Draw(null));
        Assert.Equal(initials, Sample("cleared"));

        bool Draw(string? image)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(12);
            bool decoded = window.Avatar("avatar", "Jonas Berg", image, "online", 44, square: true, track: true);
            window.EndColumn();
            window.Present();
            return decoded;
        }

        uint Sample(string name)
        {
            string png = directory.File(name + ".png");
            window.CapturePng(png);
            return PngAssertions.ReadPixels(png, width, height)[center];
        }
    }

    /// <summary>A presence pill renders a collaborator's image and falls back to initials for a missing file.</summary>
    [GuiFact]
    public void PresencePillUsesTheCurrentAvatarPath()
    {
        using var directory = new TemporaryDirectory();
        string path = directory.File("画像.png");
        File.WriteAllBytes(path, TestImages.Blue);
        const int width = 400, height = 320;
        using var window = new CuiWindow("CUI presence pill", width, height, CuiRenderer.Software, continuous: true);
        var person = new CuiLivePerson("Jonas Berg", 0, 0, 0, null, false, false, 0);

        uint initials = DrawAndSample(person, "initials");
        Assert.Equal(0xffdc641eu, DrawAndSample(person with { Avatar = path }, "image"));
        Assert.Equal(initials, DrawAndSample(person with { Avatar = directory.File("missing.png") }, "fallback"));

        uint DrawAndSample(CuiLivePerson entry, string label)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(12);
            window.PresencePill("presence", [entry], 26, 5);
            window.TrackLastControl("presence");
            window.EndColumn();
            window.Present();
            var pill = window.GetControlRectangle("presence");
            int center = (int)(pill.Y + pill.Height / 2) * width + (int)(pill.X + 18);
            string capture = directory.File(label + ".png");
            window.CapturePng(capture);
            return PngAssertions.ReadPixels(capture, width, height)[center];
        }
    }
}
