// [Documentation](../../../docs/ui/gui/files/file-picker.md)
using System.Diagnostics;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests.Packaging;

/// <summary>Drives CUI's real Zenity chooser over X11 with bounded external commands.</summary>
internal static class LinuxFileDialogInput
{
    internal static bool Available => OperatingSystem.IsLinux() && GuiAvailability.SkipReason is null &&
        new[] { "zenity", "xdotool", "xclip" }.All(name =>
            (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator)
                .Any(directory => File.Exists(Path.Combine(directory, name))));

    internal static async Task Respond(string title, string? path, bool folder)
    {
        string? id = null;
        Process? clipboard = null;
        try
        {
            var deadline = Stopwatch.StartNew();
            while (deadline.Elapsed < TimeSpan.FromSeconds(10))
            {
                var found = await Run("search", "--onlyvisible", "--name", "^" + title + "$");
                if (found.ExitCode == 0) { id = found.Output.Trim().Split('\n')[0]; break; }
                await Task.Delay(50);
            }
            Assert.False(string.IsNullOrEmpty(id), "The native chooser did not open.");
            await Checked("windowfocus", "--sync", id!);
            await Task.Delay(150);
            if (path is null) { await Checked("key", "Escape"); return; }
            if (path.Length == 0) await Checked("key", "Return");
            else
            {
                await Checked("key", "ctrl+l");
                await Checked("key", "ctrl+a");
                // A real UTF-8 clipboard owner avoids transient Unicode keyboard remapping.
                var start = new ProcessStartInfo("xclip") { RedirectStandardInput = true, UseShellExecute = false };
                foreach (string argument in new[] { "-selection", "clipboard", "-quiet" }) start.ArgumentList.Add(argument);
                clipboard = Process.Start(start) ?? throw new InvalidOperationException("Could not own the test clipboard.");
                await clipboard.StandardInput.WriteAsync(path);
                clipboard.StandardInput.Close();
                await Task.Delay(100);
                await Checked("key", "ctrl+v");
                await Task.Delay(150);
                await Checked("key", "Return");
            }
            // Folder selection first navigates into the directory; confirm with the dialog's footer.
            if (folder)
            {
                await Task.Delay(200);
                var visible = await Run("search", "--onlyvisible", "--name", "^" + title + "$");
                if (visible.ExitCode == 0)
                {
                    var geometry = await Run("getwindowgeometry", "--shell", id!);
                    var dimensions = geometry.Output.Split('\n').Where(line => line.Contains('='))
                        .Select(line => line.Split('=', 2)).ToDictionary(pair => pair[0], pair => pair[1]);
                    await Checked("mousemove", "--window", id!,
                        (int.Parse(dimensions["WIDTH"]) - 40).ToString(),
                        (int.Parse(dimensions["HEIGHT"]) - 28).ToString());
                    await Checked("click", "1");
                }
            }
            deadline.Restart();
            while (deadline.Elapsed < TimeSpan.FromSeconds(10))
            {
                var found = await Run("search", "--onlyvisible", "--name", "^" + title + "$");
                if (found.ExitCode != 0) { id = null; return; }
                await Task.Delay(50);
            }
            Assert.Fail("The native chooser did not accept the selection.");
        }
        finally
        {
            if (clipboard is not null)
            {
                if (!clipboard.HasExited) clipboard.Kill(entireProcessTree: true);
                await clipboard.WaitForExitAsync().WaitAsync(TimeSpan.FromSeconds(5));
                clipboard.Dispose();
            }
            if (id is not null)
            {
                await Run("windowfocus", id);
                await Run("key", "Escape");
            }
        }
    }

    private static Task<ProcessResult> Run(params string[] arguments) =>
        ProcessRunner.RunExecutableAsync("xdotool", arguments, timeLimit: TimeSpan.FromSeconds(5));

    private static async Task Checked(params string[] arguments)
    {
        var result = await Run(arguments);
        Assert.True(result.ExitCode == 0, result.Error);
    }
}

internal sealed class LinuxFileDialogTheoryAttribute : TheoryAttribute
{
    public LinuxFileDialogTheoryAttribute() => Skip = LinuxFileDialogInput.Available
        ? null : "Requires X11, Zenity, xdotool and xclip to exercise native file dialogs.";
}
