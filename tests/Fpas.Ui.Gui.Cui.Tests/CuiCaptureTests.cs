using Fpas.Ui.Gui.Cui.Widgets.Files;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises the native capture widgets and real Windows file input.</summary>
public sealed class CuiCaptureTests
{
    private const string BluePng = "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAAAI0lEQVR4nGOUS7nDQApgIkk1w6gG4gATkergYFQDMYDkUAIAYLIBfpuLCPgAAAAASUVORK5CYII=";
    private const string RedPng = "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAAAI0lEQVR4nGO8pKzLQApgIkk1w6gG4gATkergYFQDMYDkUAIAqr4BQidXde8AAAAASUVORK5CYII=";

    /// <summary>Shell drops keep accepted files in order and replace the avatar with a decoded image.</summary>
    [WindowsGuiFact]
    public void ShellDropFiltersPathsAndReplacesAvatar()
    {
        using var directory = new TemporaryDirectory();
        string image = directory.File("画像.png"), replacement = directory.File("replacement.png");
        string document = directory.File("document.pdf"), ignored = directory.File("ignored.txt");
        File.WriteAllBytes(image, Convert.FromBase64String(BluePng));
        File.WriteAllBytes(replacement, Convert.FromBase64String(RedPng));
        File.WriteAllText(document, "%PDF-1.0");
        File.WriteAllText(ignored, "not accepted");

        const string title = "FPas Capture shell drop";
        using var window = new CuiWindow(title, 640, 360, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        nint handle = WindowsCaptureInput.Window(title);
        Draw();
        var drop = window.GetControlRectangle("files");
        var avatar = window.GetControlRectangle("avatar");

        WindowsCaptureInput.Drop(handle, window.Size, drop, image, ignored, document);
        Assert.Equal(new[] { image, document }, Draw().Files.Paths);
        Assert.Empty(Draw().Files.Paths);
        WindowsCaptureInput.Drop(handle, window.Size, drop, image);
        Assert.Empty(Draw(enabled: false).Files.Paths);
        WindowsCaptureInput.Drop(handle, window.Size, avatar, image);
        var first = Draw().Avatar;
        Assert.True(first.HasImage);
        Assert.Equal(image, first.Path);
        Assert.True(Draw(image).Avatar.HasImage);
        string before = directory.File("before.png");
        window.CapturePng(before);

        WindowsCaptureInput.Drop(handle, window.Size, avatar, replacement);
        var second = Draw(image).Avatar;
        Assert.True(second.HasImage);
        Assert.Equal(replacement, second.Path);
        Assert.True(Draw(replacement).Avatar.HasImage);
        string after = directory.File("after.png");
        window.CapturePng(after);
        int center = (int)(avatar.Y + avatar.Height / 2) * 640 + (int)(avatar.X + avatar.Width / 2);
        Assert.NotEqual(PngAssertions.ReadPixels(before, 640, 360)[center],
            PngAssertions.ReadPixels(after, 640, 360)[center]);

        (CuiDropzoneResult Files, CuiAvatarUploadResult Avatar) Draw(string? path = null, bool enabled = true)
        {
            Assert.True(window.NextFrame());
            window.BeginColumn(12, 10);
            var files = window.Dropzone("files", "", "", "png;pdf", "", 168, false, enabled, false, true);
            var profile = window.AvatarUpload("avatar", "Maya Okafor", path, 88, enabled, false, true);
            window.EndColumn();
            window.Present();
            return (files, profile);
        }
    }

    /// <summary>The signature snapshot owns its data and separates successive strokes.</summary>
    [GuiFact]
    public void SignatureSnapshotsOwnTheirPointsAndStrokeBoundaries()
    {
        using var window = new CuiWindow("FPas Capture strokes", 380, 240, CuiRenderer.Software, continuous: true);
        Draw(-1, -1, false);
        var box = window.GetControlRectangle("signature");
        float x = box.X + 32, y = box.Y + 64;
        Draw(x, y, true);
        var first = Assert.IsType<CuiSignature>(Draw(x + 20, y + 4, true));
        Draw(x + 20, y + 4, false);
        Draw(x + 40, y + 24, true);
        var second = Assert.IsType<CuiSignature>(Draw(x + 70, y + 25, true));

        Assert.Equal(new[] { 2 }, first.StrokeEnds);
        Assert.Equal(new[] { 2, 4 }, second.StrokeEnds);
        Assert.Equal(first.Coordinates, second.Coordinates[..4]);
        Assert.Equal(new[] { 32f, 64f, 52f, 68f }, first.Coordinates);

        CuiSignature? Draw(float px, float py, bool pressed)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(px, py);
            window.SetLeftButton(pressed);
            window.BeginColumn(12);
            var result = window.SignaturePad("signature", "", 180, 0, enabled: true, focusClear: false,
                track: true, readChanges: true, clearId: null, out _);
            window.EndColumn();
            window.Present();
            return result;
        }
    }

    /// <summary>Tracks the Clear button independently of the signature pad.</summary>
    [GuiFact]
    public void SignatureClearCanBeTrackedWithoutTrackingThePad()
    {
        using var window = new CuiWindow("Signature clear", 380, 240, CuiRenderer.Software, continuous: true);
        for (int frame = 0; frame < 2; frame++)
        {
            Assert.True(window.NextFrame());
            if (frame == 1)
            {
                window.MovePointer(44, 72);
                window.SetLeftButton(true);
            }
            window.BeginColumn(12);
            window.SignaturePad("signature", "", 180, 0, enabled: true, focusClear: false,
                track: false, readChanges: false, clearId: "clear", out bool clearVisible);
            window.EndColumn();
            window.Present();
            Assert.Equal(frame == 1, clearVisible);
        }

        Assert.True(window.GetControlRectangle("clear").Width > 0);
    }

    /// <summary>The owner window's common dialog returns a Unicode file selection and handles cancellation.</summary>
    [WindowsGuiFact]
    public void WindowsFileDialogSelectsAndCancels()
    {
        using var directory = new TemporaryDirectory();
        string image = directory.File("画像.png");
        File.WriteAllBytes(image, Convert.FromBase64String(BluePng));
        const string title = "FPas Capture file dialog";
        using var window = new CuiWindow(title, renderer: CuiRenderer.Software, continuous: true);
        nint handle = WindowsCaptureInput.Window(title);

        Assert.Equal(image, Choose(image));
        Assert.Null(Choose(null));

        string? Choose(string? path)
        {
            Task driver = Task.Run(() => WindowsCaptureInput.RespondToDialog(handle, path));
            string? result = window.OpenFile("Select test image", "Images", "png");
            driver.GetAwaiter().GetResult();
            return result;
        }
    }
}
