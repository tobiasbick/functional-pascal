using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;

namespace Fpas.IntegrationTests.Refactoring;

internal static class RenameFixture
{
    internal static TemporaryDirectory Create(SdkPackageFixture sdk, string source)
    {
        var directory = sdk.CreateProject(); File.WriteAllText(directory.File("Game/Bridge.fpas"), source); return directory;
    }
    internal static async Task<WorkspaceAnalysis> Analyze(TemporaryDirectory directory, string context = "Game/Game.fpasproj")
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var workspace = await WorkspaceLoader.LoadAsync(new(directory.File(context)), deadline.Token);
        var analysis = WorkspaceAnalysis.Analyze(workspace, deadline.Token);
        Assert.True(analysis.Success, string.Join("; ", workspace.Issues.Select(issue => issue.Message).Concat(analysis.Projects.SelectMany(project => project.Result?.Diagnostics ?? []).Select(diagnostic => diagnostic.Message))));
        return analysis;
    }
    internal static SourceSelection Select(TemporaryDirectory directory, string source, string marker, string file = "Game/Bridge.fpas")
    {
        var offset = source.IndexOf(marker, StringComparison.Ordinal); Assert.True(offset >= 0, marker);
        var lines = source[..offset].Split('\n'); return new(directory.File(file), lines.Length, lines[^1].Length + 1);
    }
    internal static Task<ProcessResult> Cli(TemporaryDirectory directory, params string[] args)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "refactor", .. args], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(3));
}
