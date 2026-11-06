using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed partial class PublishingTests
{
    [Fact]
    public async Task Publish_timeout_reaps_owned_build_children_and_preserves_the_previous_delivery()
    {
        using var directory = new TemporaryDirectory(); var project = await Create(directory);
        var destination = directory.File("output");
        var first = await Cli(directory, "publish", project, "--output", destination, "--mode", "apphost", "--report", "json");
        Assert.True(first.ExitCode == 0, first.Output + first.Error);
        var before = Directory.GetFiles(destination).Order().Select(File.ReadAllBytes).ToArray();
        using var waiting = await Fpas.IntegrationTests.Build.Support.WaitingBuild.Prepare(project);
        var timed = await Cli(directory, "publish", project, "--output", destination, "--mode", "apphost", "--timeout", "30", "--report", "json");
        Assert.Equal(1, timed.ExitCode);
        using var report = JsonDocument.Parse(timed.Output); Assert.Equal("timeout", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.True(File.Exists(waiting.Ready), "The deadline expired before the blocking build helper started. " + timed.Output + timed.Error);
        var pid = int.Parse(File.ReadAllText(waiting.Ready), System.Globalization.CultureInfo.InvariantCulture);
        try { using var child = System.Diagnostics.Process.GetProcessById(pid); Assert.True(child.HasExited); } catch (ArgumentException) { }
        Assert.Equal(before, Directory.GetFiles(destination).Order().Select(File.ReadAllBytes).ToArray());
        Assert.Empty(Directory.GetDirectories(directory.Path, ".fpas-publish-*"));
    }

    [Fact]
    public async Task Republish_removes_stale_outputs_and_preserves_previous_delivery_on_failure_or_conflict()
    {
        using var directory = new TemporaryDirectory(); var project = await Create(directory);
        var destination = directory.File("output");
        var preview = await Cli(directory, "publish", project, "--output", destination, "--mode", "apphost", "--dry-run", "--report", "json");
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error); Assert.False(Directory.Exists(destination));
        var first = await Cli(directory, "publish", project, "--output", destination, "--mode", "apphost", "--report", "json");
        Assert.True(first.ExitCode == 0, first.Output + first.Error);
        Assert.True(File.Exists(Path.Combine(destination, "Shared.dll")));
        var changed = await Cli(directory, "publish", project, "--output", destination, "--mode", "single-file", "--report", "json");
        Assert.True(changed.ExitCode == 0, changed.Output + changed.Error); Assert.False(File.Exists(Path.Combine(destination, "Shared.dll")));
        var before = Directory.GetFiles(destination).Order().Select(File.ReadAllBytes).ToArray();
        File.WriteAllText(directory.File("App/main.fpas"), "program Broken; begin Unknown(); end; end program;");
        var failed = await Cli(directory, "publish", project, "--output", destination, "--mode", "apphost", "--report", "json");
        Assert.Equal(1, failed.ExitCode);
        Assert.Equal(before, Directory.GetFiles(destination).Order().Select(File.ReadAllBytes).ToArray());
        File.WriteAllText(Path.Combine(destination, "keep.txt"), "owned by the user");
        var conflict = await Cli(directory, "publish", project, "--output", destination, "--report", "json");
        Assert.Equal(1, conflict.ExitCode);
        using var report = JsonDocument.Parse(conflict.Output);
        Assert.Equal("output_conflict", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal("owned by the user", File.ReadAllText(Path.Combine(destination, "keep.txt")));
        Assert.Empty(Directory.GetDirectories(directory.Path, ".fpas-publish-*"));
    }

    [Theory]
    [InlineData("--timeout", "0")]
    [InlineData("--mode", "unknown")]
    [InlineData("--runtime", "os-unsupported")]
    [InlineData("--key-file", "absent.snk")]
    public async Task Invalid_publish_options_have_structured_failures(string option, string value)
    {
        using var directory = new TemporaryDirectory(); var project = await Create(directory);
        var result = await Cli(directory, "publish", project, "--output", directory.File("output"), option, value, "--report", "json");
        Assert.Equal(2, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(Directory.Exists(directory.File("output")));
    }
}
