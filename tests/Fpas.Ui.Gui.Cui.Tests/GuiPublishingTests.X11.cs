using System.Diagnostics;
using System.Text;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

public sealed partial class GuiPublishingTests
{
    /// <summary>Drives the real public gallery through X11 pointer, keyboard, resize and close events.</summary>
    [X11GuiFact]
    public async Task PackagedGalleryAcceptsOperatingSystemInput()
    {
        using var fixture = new GuiPackageFixture();
        await fixture.Prepare();
        string title = "FPAS X11 " + Guid.NewGuid().ToString("N");
        string capture = Path.Combine(fixture.Artifacts, "x11-input.png");
        var start = new ProcessStartInfo(TestEnvironment.DotnetHost)
        {
            WorkingDirectory = Directory.CreateDirectory(fixture.File("outside-project")).FullName,
            RedirectStandardOutput = true, RedirectStandardError = true
        };
        foreach (var (key, value) in fixture.Environment)
            if (value is null) start.Environment.Remove(key); else start.Environment[key] = value;
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        foreach (string argument in new[] { fixture.File($"consumer/bin/{configuration}/net10.0/GalleryProbe.dll"),
            "observe", title, capture }) start.ArgumentList.Add(argument);
        using var process = Process.Start(start)!;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var token = deadline.Token;
        Task<string> errors = process.StandardError.ReadToEndAsync(token);
        var transcript = new StringBuilder();
        Frame? latest = null;
        try
        {
            await Read(_ => true);
            using var input = new X11Input(title);
            await Click("browse");
            await Read(frame => frame.Has("save"));
            await Click("format-bold");
            await Read(frame => frame.Find("format-bold").Value == "true");
            await Click("theme");
            await Read(frame => frame.Dark);
            input.Resize(520, 900);
            await Read(frame => frame.Width == 520 && frame.Height == 900 && frame.Dark &&
                frame.Find("format-bold").Value == "true");
            await Click("drawer-open");
            await Read(frame => frame.Controls.Any(control =>
                control.Id == "navigation" && control.Key == "overlays"));
            await Click("navigation", "overlays");
            await Read(frame => frame.Has("sheet-open"));
            // CUI retains the input-blocking scrim while the drawer slides out.
            await Task.Delay(TimeSpan.FromMilliseconds(500), token);
            await Click("sheet-open");
            await Read(frame => frame.Has("sheet/close") && frame.Find("sheet/close").Focused);
            await Key("Escape", true); await Key("Escape", false);
            await Read(frame => !frame.Has("sheet/close") && frame.Find("sheet-open").Focused);
            input.Close();
            await process.WaitForExitAsync(token);
            Assert.Equal(0, process.ExitCode);
            Assert.Empty(await errors);
            PngAssertions.HasRenderedContent(capture, 520, 900);

            async Task Click(string id, string key = "")
            {
                var control = latest!.Find(id, key);
                long before = latest.Id;
                input.Move(control.X + control.Width / 2, control.Y + control.Height / 2);
                await Read(frame => frame.Id > before);
                before = latest!.Id; input.Button(true); await Read(frame => frame.Id > before);
                before = latest!.Id; input.Button(false); await Read(frame => frame.Id > before);
            }
            async Task Key(string key, bool down)
            {
                long before = latest!.Id;
                input.Key(key, down);
                await Read(frame => frame.Id > before);
            }
        }
        finally
        {
            File.WriteAllText(Path.Combine(fixture.Artifacts, "x11-input.log"), transcript.ToString());
            if (!process.HasExited) { process.Kill(entireProcessTree: true); await process.WaitForExitAsync(); }
        }

        async Task Read(Func<Frame, bool> matches)
        {
            if (latest is not null && matches(latest)) return;
            while (true)
            {
                string line = await process.StandardOutput.ReadLineAsync(token)
                    ?? throw new InvalidOperationException("The GUI ended before the next frame: " + await errors);
                transcript.AppendLine(line);
                if (!line.StartsWith("FRAME ", StringComparison.Ordinal)) continue;
                string[] header = line.Split(' ');
                var controls = new List<Control>();
                var text = new List<string>();
                while ((line = await process.StandardOutput.ReadLineAsync(token) ?? "") != "END")
                {
                    if (line.Length == 0) throw new InvalidOperationException("The GUI ended within a frame.");
                    transcript.AppendLine(line);
                    string[] parts = line.Split(' ');
                    if (parts[0] == "TEXT") { text.Add(Decoded(parts[1])); continue; }
                    controls.Add(new(Decoded(parts[1]), Decoded(parts[2]), Decoded(parts[3]),
                        int.Parse(parts[4]), int.Parse(parts[5]), int.Parse(parts[6]), int.Parse(parts[7]), bool.Parse(parts[8])));
                }
                latest = new(long.Parse(header[1]), int.Parse(header[2]), int.Parse(header[3]), bool.Parse(header[4]), controls, text);
                if (matches(latest)) return;
            }
        }
    }

    private static string Decoded(string value) => Encoding.UTF8.GetString(Convert.FromBase64String(value));
    private sealed record Control(string Id, string Key, string Value, int X, int Y, int Width, int Height, bool Focused);
    private sealed record Frame(long Id, int Width, int Height, bool Dark, List<Control> Controls, List<string> Text)
    {
        internal bool Has(string id) => Controls.Any(control => control.Id == id);
        internal Control Find(string id, string key = "") => Assert.Single(Controls, control => control.Id == id && control.Key == key);
    }
}
