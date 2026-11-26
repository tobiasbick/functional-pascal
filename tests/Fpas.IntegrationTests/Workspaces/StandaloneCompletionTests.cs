// [Documentation](../../../docs/pascal/program-structure/cli.md#completion-and-signature-help)
using Fpas.IntegrationTests.Documentation;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Workspaces;

/// <summary>Exercises compiler completion through the CLI on files without a project.</summary>
public sealed class StandaloneCompletionTests
{
    /// <summary>Checks incomplete member access, the replacement span and source-only coverage.</summary>
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Completes_programs_and_units_without_a_project(bool unit)
    {
        using var directory = new TemporaryDirectory(Path.Combine(DocumentationLinks.Repository, ".temp-data"));
        const string file = "Grüße probe.fpas";
        var path = directory.File(file);
        var text = unit ? """
            // Completion in a unit.
            unit Probe;
            uses System.Console as Console;
            procedure Show();
            begin
              Console.WriteL;
            end procedure;
            end unit;
            """ : """
            program Probe;
            uses System.Console as Console;
            begin
              Console.
            end;
            end program;
            """;
        File.WriteAllText(path, text);

        var result = await Complete(directory, file, unit ? "6" : "4", unit ? "17" : "11", "--diagnostic-limit", "0");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var report = json.RootElement;
        Assert.Equal("ok", report.GetProperty("status").GetString());
        Assert.Equal(path, report.GetProperty("context").GetString());
        Assert.NotEmpty(report.GetProperty("snapshotId").GetString()!);
        Assert.Empty(report.GetProperty("coverage").GetProperty("requested").EnumerateArray());
        Assert.False(report.TryGetProperty("preparation", out _));
        Assert.False(report.GetProperty("selection").TryGetProperty("project", out _));
        Assert.Empty(report.GetProperty("diagnostics").EnumerateArray());
        Assert.True(report.GetProperty("diagnosticPage").GetProperty("totalCount").GetInt32() > 0);
        var completion = report.GetProperty("result").GetProperty("complete");
        Assert.Equal("member", completion.GetProperty("context").GetString());
        Assert.Contains(completion.GetProperty("items").EnumerateArray(), item => item.GetProperty("name").GetString() == "WriteLine");
        var replacement = completion.GetProperty("replacement");
        Assert.Equal(path, replacement.GetProperty("path").GetString());
        Assert.Equal(11, replacement.GetProperty("column").GetInt32());
        Assert.Equal(unit ? 6 : 0, replacement.GetProperty("length").GetInt32());
        Assert.Equal(path, Assert.Single(Directory.EnumerateFileSystemEntries(directory.Path)));
        Assert.Equal(text, File.ReadAllText(path));
    }

    /// <summary>Checks that file and cursor selection errors stay structured and stale snapshots fail.</summary>
    [Fact]
    public async Task Validates_the_file_position_and_expected_snapshot()
    {
        using var directory = new TemporaryDirectory(Path.Combine(DocumentationLinks.Repository, ".temp-data"));
        const string file = "Probe.fpas";
        var path = directory.File(file);
        File.WriteAllText(path, "program Probe;\nbegin\nend;\nend program;\n");
        var initial = await Complete(directory, file, "2", "1", "--file", file);
        Assert.True(initial.ExitCode == 0, initial.Output + initial.Error);
        using var report = JsonDocument.Parse(initial.Output);
        var snapshot = report.RootElement.GetProperty("snapshotId").GetString()!;

        AssertError(await Complete(directory, file, "2", "1", "--file", "Other.fpas"), 2, "invalid_argument");
        AssertError(await Complete(directory, file, "2", "10"), 2, "invalid_argument");
        File.AppendAllText(path, "// Changed\n");
        AssertError(await Complete(directory, file, "2", "1", "--expected-snapshot", snapshot), 1, "inputs_changed");

        static void AssertError(ProcessResult result, int exit, string code)
        {
            Assert.True(result.ExitCode == exit, result.Output + result.Error);
            using var json = JsonDocument.Parse(result.Output);
            Assert.Equal(code, json.RootElement.GetProperty("error").GetProperty("code").GetString());
        }
    }

    private static Task<ProcessResult> Complete(TemporaryDirectory directory, string file, string line, string column, params string[] options)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "complete", file, "--line", line, "--column", column, .. options],
            workingDirectory: directory.Path);
}
