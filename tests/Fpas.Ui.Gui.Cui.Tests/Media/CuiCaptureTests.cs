// [Documentation](../../../docs/ui/gui/capture/README.md)
using Fpas.Ui.Gui.Cui.Widgets.Files;
using Fpas.Tests.Support;
using Fpas.Ui.Gui.Cui.Tests.Windowing;

namespace Fpas.Ui.Gui.Cui.Tests.Media;

/// <summary>Exercises the native capture widgets and real Windows file input.</summary>
public sealed class CuiCaptureTests
{
    /// <summary>Explicit empty text and icon options suppress their native defaults.</summary>
    [GuiFact]
    public void DropzoneDistinguishesOmittedAndEmptyOptions()
    {
        using var directory = new TemporaryDirectory();
        using var window = new CuiWindow("FPas Capture dropzone options", 360, 240, CuiRenderer.Software, continuous: true);
        window.SetReducedMotion(true);
        uint[] defaults = Capture("default.png");

        Assert.False(defaults.AsSpan().SequenceEqual(Capture("no-title.png", title: "")));
        Assert.False(defaults.AsSpan().SequenceEqual(Capture("no-description.png", description: "")));
        Assert.False(defaults.AsSpan().SequenceEqual(Capture("no-icon.png", icon: "")));

        uint[] Capture(string file, string? title = null, string? description = null, string? icon = null)
        {
            for (int frame = 0; frame < 2; ++frame)
            {
                Assert.True(window.NextFrame());
                window.MovePointer(-1, -1);
                window.Dropzone("files", title, description, "", icon, 0, false, true, false, true,
                    widthKind: 2, width: 176);
                window.Present();
            }

            var bounds = window.GetControlRectangle("files");
            Assert.Equal(176f, bounds.Width);
            Assert.Equal(168f, bounds.Height);
            string path = directory.File(file);
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, 360, 240);
        }
    }

    /// <summary>Shell drops keep accepted files in order and replace the avatar with a decoded image.</summary>
    [WindowsGuiFact]
    public void ShellDropFiltersPathsAndReplacesAvatar()
    {
        using var directory = new TemporaryDirectory();
        string image = directory.File("画像.png"), replacement = directory.File("replacement.png");
        string document = directory.File("document.pdf"), ignored = directory.File("ignored.txt");
        File.WriteAllBytes(image, TestImages.Blue);
        File.WriteAllBytes(replacement, TestImages.Red);
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
            var result = window.SignaturePad("signature", null, 1, 0, 180, default, enabled: true, focusClear: false,
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
            window.SignaturePad("signature", null, 1, 0, 180, default, enabled: true, focusClear: false,
                track: false, readChanges: false, clearId: "clear", out bool clearVisible);
            window.EndColumn();
            window.Present();
            Assert.Equal(frame == 1, clearVisible);
        }

        Assert.True(window.GetControlRectangle("clear").Width > 0);
    }

    /// <summary>Resolves native percentage width and distinguishes a blank hint from the default.</summary>
    [GuiFact]
    public void SignatureUsesNativeWidthAndOptionalPlaceholder()
    {
        const int width = 400, height = 240;
        using var window = new CuiWindow("Signature options", width, height, CuiRenderer.Software, continuous: true);
        using var directory = new TemporaryDirectory();
        window.SetReducedMotion(true);
        uint[] standard = Draw("standard", null);
        uint[] blank = Draw("blank", "");
        Assert.False(standard.SequenceEqual(blank));

        uint[] Draw(string name, string? hint)
        {
            Assert.True(window.NextFrame());
            window.MovePointer(-1, -1);
            window.BeginColumn(12);
            window.SignaturePad("signature", hint, 3, 50, -1, default, enabled: true, focusClear: false,
                track: true, readChanges: false, clearId: null, out bool clearVisible);
            window.EndColumn();
            window.Present();
            Assert.False(clearVisible);
            var bounds = window.GetControlRectangle("signature");
            Assert.Equal(188, bounds.Width);
            Assert.Equal(180, bounds.Height);
            string path = directory.File(name + ".png");
            window.CapturePng(path);
            return PngAssertions.ReadPixels(path, width, height);
        }
    }

    /// <summary>The owner window's common dialog returns a Unicode file selection and handles cancellation.</summary>
    [WindowsGuiFact]
    public void WindowsFileDialogSelectsAndCancels()
    {
        using var directory = new TemporaryDirectory();
        string image = directory.File("画像.png");
        File.WriteAllBytes(image, TestImages.Blue);
        const string title = "FPas Capture file dialog";
        using var window = new CuiWindow(title, renderer: CuiRenderer.Software, continuous: true);
        nint handle = WindowsCaptureInput.Window(title);

        Assert.Equal(image, Choose(image));
        Assert.Null(Choose(null));

        string? Choose(string? path)
        {
            Task driver = Task.Run(() => WindowsFileDialogInput.Respond(handle, path));
            string? result = window.OpenFile("Select test image", "Images", "png");
            driver.GetAwaiter().GetResult();
            return result;
        }
    }
}
