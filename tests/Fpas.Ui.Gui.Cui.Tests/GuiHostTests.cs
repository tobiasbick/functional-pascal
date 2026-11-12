using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises the FPAS MVU owner loop with real CUI presentation and native interactions.</summary>
public sealed class GuiHostTests
{
    /// <summary>Builds the public gallery and verifies its visible pages and controls.</summary>
    /// <param name="renderer">The renderer that must be selected by the FPAS host.</param>
    [GuiTheory]
    [InlineData("software")]
    [InlineData("opengl")]
    public async Task FpasHostOwnsTypedTransitionsAndNativeFrames(string renderer)
    {
        using var directory = new TemporaryDirectory();
        string version = "0.1.0-host." + Guid.NewGuid().ToString("N");
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        string feed = Directory.CreateDirectory(directory.File("feed")).FullName;
        var pack = await ProcessRunner.RunAsync(["pack", TestEnvironment.Metadata("FpasSdkProject"), "--no-build",
            "-c", configuration, "--output", feed, "-p:PackageVersion=" + version], timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(pack.ExitCode == 0, pack.Output + pack.Error);
        string fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "host");
        foreach (string source in Directory.GetFiles(fixture, "*", SearchOption.AllDirectories))
        {
            string target = directory.File(Path.GetRelativePath(fixture, source));
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            File.Copy(source, target);
        }
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "host-sdk"), new XAttribute("value", feed)))))
            .Save(directory.File("NuGet.Config"));
        foreach (string path in Directory.GetFiles(directory.Path, "*.fpasproj", SearchOption.AllDirectories))
        {
            var project = XDocument.Load(path);
            project.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + version);
            foreach (var reference in project.Descendants("ProjectReference").Where(reference =>
                reference.Attribute("Include")!.Value.EndsWith("Fpas.Ui.Gui.Cui.csproj", StringComparison.Ordinal)))
                reference.SetAttributeValue("Include", TestEnvironment.Metadata("FpasCuiProject"));
            foreach (var reference in project.Descendants("ProjectReference").Where(reference =>
                reference.Attribute("Include")!.Value.EndsWith("Fpas.Processes.csproj", StringComparison.Ordinal)))
                reference.SetAttributeValue("Include", TestEnvironment.Metadata("FpasProcessesProject"));
            project.Save(path);
        }
        var environment = new Dictionary<string, string?> { ["NUGET_PACKAGES"] = directory.File("packages"), ["LD_LIBRARY_PATH"] = null };
        foreach (string project in new[] { "src/Fpas.Ui.Mvu/Fpas.Ui.Mvu.fpasproj", "src/Fpas.Ui/Fpas.Ui.fpasproj",
            "src/Fpas.Ui.Gui/Fpas.Ui.Gui.fpasproj",
            "examples/ui/gui/gallery/GuiWidgetGallery.fpasproj" })
        {
            var build = await ProcessRunner.RunAsync(["build", directory.File(project), "-c", configuration,
                "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
                workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
            Assert.True(build.ExitCode == 0, project + ": " + build.Output + build.Error);
        }
        CheckDocumentation(directory.File($"src/Fpas.Ui.Mvu/bin/{configuration}/net10.0/Fpas.Ui.Mvu.xml"),
            "M:Fpas.Ui.Mvu.Application.Focus``1(", ["Id"], ["TMsg"]);
        CheckDocumentation(directory.File($"src/Fpas.Ui/bin/{configuration}/net10.0/Fpas.Ui.xml"),
            "M:Fpas.Ui.Elements.CachedSubview``2(", ["Id", "Dependencies", "Build", "Equal"], ["TDeps", "TMsg"]);
        CheckDocumentation(directory.File($"src/Fpas.Ui.Gui/bin/{configuration}/net10.0/Fpas.Ui.Gui.xml"),
            "M:Fpas.Ui.Gui.Application.Start``2(", ["InitialModel", "Update", "View", "Settings", "ThemeFor"], ["TModel", "TMsg"]);
        File.Copy(directory.File("PublishedGallery.fpas"), directory.File("examples/ui/gui/gallery/main.fpas"), overwrite: true);
        foreach (string helper in Directory.GetFiles(directory.Path, "*.fpas"))
            if (Path.GetFileName(helper) != "PublishedGallery.fpas")
                File.Copy(helper, directory.File("examples/ui/gui/gallery/" + Path.GetFileName(helper)));
        var galleryBuild = await ProcessRunner.RunAsync(["build", directory.File("examples/ui/gui/gallery/GuiWidgetGallery.fpasproj"), "-c", configuration,
            "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(galleryBuild.ExitCode == 0, galleryBuild.Output + galleryBuild.Error);
        string galleryCapture = directory.File("gallery.png");
        var gallery = await ProcessRunner.RunAsync([directory.File($"examples/ui/gui/gallery/bin/{configuration}/net10.0/GuiWidgetGallery.dll"), renderer, galleryCapture],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromSeconds(90));
        Assert.True(gallery.ExitCode == 0, gallery.Output + gallery.Error);
        Assert.Equal("GUI gallery verified: " + renderer + Environment.NewLine, gallery.Output);
        PngAssertions.HasRenderedContent(galleryCapture, 1000, 760);
        CheckWheelFocus(galleryCapture);
        CheckRulerFocus(galleryCapture);
        CheckDateFocus(galleryCapture);
        CheckColorFocus(galleryCapture);
        CheckSignatureInk(galleryCapture);
        CheckChecklistFocus(galleryCapture);
        CheckHoverLinkFocus(galleryCapture);
        CheckReaderMarker(galleryCapture);
        string? artifacts = Environment.GetEnvironmentVariable("FPAS_GUI_TEST_ARTIFACTS");
        foreach (var (page, width, height) in new[]
        {
            ("actions", 1000, 760), ("status", 1000, 760),
            ("actions-account", 1000, 1040), ("user-menu", 1000, 1040), ("selection", 1100, 1360),
            ("actions-complete", 1000, 1440), ("swipe-revealed", 1000, 1440),
            ("speed-dial", 1000, 1440), ("multi-select", 1000, 1440), ("multi-select-open", 1000, 1440),
            ("date-range", 1000, 1000), ("date-range-presets", 1000, 1000), ("date-range-stay", 1000, 1000),
            ("due-date", 1000, 1700), ("due-date-open", 1000, 1700), ("due-date-focused", 1000, 1700),
            ("brand-color", 1000, 1700), ("brand-color-open", 1000, 1700), ("brand-color-focused", 1000, 1700),
            ("emoji-message", 1000, 800), ("emoji-open", 1000, 800),
            ("time-wheels-dial", 1000, 1280), ("time-24-hour", 1000, 1280), ("time-12-hour", 1000, 1280),
            ("wheel-focused", 1000, 1280),
            ("ruler-histogram", 1000, 1600), ("ruler-focused", 1000, 1600), ("histogram-adjusted", 1000, 1600),
            ("navigation", 1000, 900), ("navigation-compact", 360, 900), ("toasts", 1000, 760),
            ("tooltips", 1000, 900), ("popover", 1000, 900), ("confirm-popover", 1000, 900),
            ("text-fields", 1000, 760), ("text-fields-compact", 360, 760),
            ("tags-codes-strength", 1000, 1000),
            ("pickers", 1000, 1200), ("pickers-search", 1000, 1200), ("pickers-bound", 1000, 1200),
            ("text-affixes", 1000, 760), ("edit-profile", 1000, 900),
            ("disclosure", 1100, 1100), ("disclosure-dark", 1100, 1100), ("disclosure-narrow", 650, 1250),
            ("capture", 1000, 1000), ("capture-signed", 1000, 1000), ("capture-cleared", 1000, 1000),
            ("disclosure-feedback", 1100, 1500), ("checklist-focused", 1100, 1500),
            ("hover-card-open", 1100, 1500), ("hover-link-focused", 1100, 1500),
            ("reader", 1100, 850), ("reader-jump", 1100, 850), ("reader-scrolled", 1100, 850),
            ("workspace", 1100, 1700), ("overview-narrow", 500, 1700), ("project-delete", 1100, 1700),
            ("notification-switches", 1100, 1700), ("disclosure-undo", 1100, 1800),
            ("disclosure-undo-toast", 1100, 1800),
            ("accordion", 1100, 760), ("audio", 1201, 800), ("audio-wrap", 900, 800),
            ("feedback", 1100, 1700), ("feedback-dark", 1100, 1700), ("feedback-narrow", 500, 1800)
        })
        {
            string pageCapture = galleryCapture + "." + page + ".png";
            PngAssertions.HasRenderedContent(pageCapture, width, height);
            if (!string.IsNullOrWhiteSpace(artifacts))
            {
                string target = Directory.CreateDirectory(Path.Combine(artifacts, "host-gallery", renderer)).FullName;
                File.Copy(pageCapture, Path.Combine(target, page + ".png"), overwrite: true);
            }
        }
    }

    private static void CheckReaderMarker(string capture)
    {
        const int width = 1100, height = 850, railX = 860;
        var rows = new[] { 137, 166, 195 };
        var states = new[] { "reader", "reader-jump", "reader-scrolled" };
        for (int state = 0; state < states.Length; ++state)
        {
            var pixels = PngAssertions.ReadPixels(capture + "." + states[state] + ".png", width, height);
            for (int row = 0; row < rows.Length; ++row)
            {
                uint red = pixels[rows[row] * width + railX] & 0xff;
                Assert.True((red < 64) == (row == state),
                    $"The Reader rail marker is in the wrong row for {states[state]}.");
            }
        }
    }

    private static void CheckHoverLinkFocus(string capture)
    {
        const int width = 1100, height = 1500;
        var normal = PngAssertions.ReadPixels(capture + ".disclosure-feedback.png", width, height);
        var focused = PngAssertions.ReadPixels(capture + ".hover-link-focused.png", width, height);
        const int left = 349, top = 243, areaWidth = 68, areaHeight = 28;
        bool changed = false;
        for (int y = top; y < top + areaHeight; ++y)
        {
            int start = y * width + left;
            changed |= !normal.AsSpan(start, areaWidth).SequenceEqual(focused.AsSpan(start, areaWidth));
        }
        Assert.True(changed, "The link must show its keyboard focus ring.");
    }

    private static void CheckChecklistFocus(string capture)
    {
        const int width = 1100, height = 1500;
        var normal = PngAssertions.ReadPixels(capture + ".disclosure-feedback.png", width, height);
        var focused = PngAssertions.ReadPixels(capture + ".checklist-focused.png", width, height);
        int normalGreen = 0, focusedGreen = 0;
        bool changed = false;
        for (int y = 1310; y < 1350; ++y)
        {
            for (int x = 295; x < 330; ++x)
            {
                int index = y * width + x;
                if (IsSuccess(normal[index])) normalGreen++;
                if (IsSuccess(focused[index])) focusedGreen++;
                changed |= normal[index] != focused[index];
            }
        }
        Assert.True(normalGreen > 250 && focusedGreen == normalGreen, "Keyboard focus must preserve the green check.");
        Assert.True(changed, "Keyboard focus must add a visible ring.");

        static bool IsSuccess(uint pixel)
        {
            uint red = pixel & 255, green = (pixel >> 8) & 255, blue = (pixel >> 16) & 255;
            return green > 2 * red && green > blue;
        }
    }

    private static void CheckSignatureInk(string capture)
    {
        const int width = 1000, height = 1000;
        var empty = PngAssertions.ReadPixels(capture + ".capture.png", width, height);
        var signed = PngAssertions.ReadPixels(capture + ".capture-signed.png", width, height);
        var cleared = PngAssertions.ReadPixels(capture + ".capture-cleared.png", width, height);
        const int ink = 795 * width + 396;
        Assert.NotEqual(empty[ink], signed[ink]);
        Assert.Equal(empty[ink], cleared[ink]);
    }

    private static void CheckWheelFocus(string capture)
    {
        const int width = 1000, height = 1280;
        uint[] atRest = PngAssertions.ReadPixels(capture + ".time-wheels-dial.png", width, height);
        uint[] focused = PngAssertions.ReadPixels(capture + ".wheel-focused.png", width, height);
        // Sample the top of the month wheel's focus ring and the caption inside it.
        const int ring = 973 * width + 401;
        Assert.False(atRest.AsSpan(ring, 56).SequenceEqual(focused.AsSpan(ring, 56)),
            "Focusing the month wheel must draw a focus ring.");
        for (int y = 1054; y <= 1065; ++y)
        {
            int start = y * width + 401;
            Assert.True(atRest.AsSpan(start, 56).SequenceEqual(focused.AsSpan(start, 56)),
                "The selected month must remain visible when the wheel receives keyboard focus.");
        }
    }

    private static void CheckRulerFocus(string capture)
    {
        const int width = 1000, height = 1600;
        uint[] atRest = PngAssertions.ReadPixels(capture + ".ruler-histogram.png", width, height);
        uint[] focused = PngAssertions.ReadPixels(capture + ".ruler-focused.png", width, height);
        // Sample the top edge of the Temperature focus ring and the tape beneath it.
        const int ring = 1499 * width + 400;
        Assert.False(atRest.AsSpan(ring, 90).SequenceEqual(focused.AsSpan(ring, 90)),
            "Focusing the ruler must draw a focus ring.");
        for (int y = 1505; y <= 1532; ++y)
        {
            int start = y * width + 400;
            Assert.True(atRest.AsSpan(start, 90).SequenceEqual(focused.AsSpan(start, 90)),
                "The ruler's ticks and marker must remain visible with keyboard focus.");
        }
    }

    private static void CheckDateFocus(string capture)
    {
        const int width = 1000, height = 1700;
        uint[] atRest = PngAssertions.ReadPixels(capture + ".due-date.png", width, height);
        uint[] focused = PngAssertions.ReadPixels(capture + ".due-date-focused.png", width, height);
        // Sample the horizontal edge of the Due date field's keyboard focus ring.
        const int ring = 1124 * width + 340;
        Assert.False(atRest.AsSpan(ring, 110).SequenceEqual(focused.AsSpan(ring, 110)),
            "The focused Due date field must display a focus ring.");
    }

    private static void CheckColorFocus(string capture)
    {
        const int width = 1000, height = 1700;
        uint[] atRest = PngAssertions.ReadPixels(capture + ".brand-color.png", width, height);
        uint[] focused = PngAssertions.ReadPixels(capture + ".brand-color-focused.png", width, height);
        const int ring = 1200 * width + 334;
        Assert.False(atRest.AsSpan(ring, 50).SequenceEqual(focused.AsSpan(ring, 50)),
            "The focused color field must display a focus ring.");
    }

    private static void CheckDocumentation(string path, string methodPrefix, string[] parameters, string[] typeParameters)
    {
        var member = Assert.Single(XDocument.Load(path).Descendants("member"), item =>
            ((string?)item.Attribute("name"))?.StartsWith(methodPrefix, StringComparison.Ordinal) == true);
        Assert.Equal(parameters, member.Elements("param").Select(item => (string?)item.Attribute("name")));
        Assert.Equal(typeParameters, member.Elements("typeparam").Select(item => (string?)item.Attribute("name")));
        foreach (var item in member.Elements().Where(item => item.Name.LocalName is "summary" or "param" or "typeparam" or "returns"))
            Assert.False(string.IsNullOrWhiteSpace(item.Value), path + ": " + item.Name);
        Assert.NotNull(member.Element("summary"));
        Assert.NotNull(member.Element("returns"));
        Assert.False(member.Element("summary")!.Value.StartsWith('/'));
    }
}
