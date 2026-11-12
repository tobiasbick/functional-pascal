using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises image replacement and initials in CUI's standalone avatar.</summary>
public sealed class CuiAvatarTests
{
    /// <summary>A model-owned path replaces the native initials and clearing it restores them.</summary>
    [GuiFact]
    public void AvatarReplacesAndClearsAnImage()
    {
        using var directory = new TemporaryDirectory();
        string path = directory.File("画像.png");
        File.WriteAllBytes(path, Convert.FromBase64String(
            "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAAAI0lEQVR4nGOUS7nDQApgIkk1w6gG4gATkergYFQDMYDkUAIAYLIBfpuLCPgAAAAASUVORK5CYII="));
        const int width = 320, height = 200;
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
}
