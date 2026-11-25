// [Documentation](../../../docs/pascal/program-structure/cli.md#check-one-source-file)
using Fpas.IntegrationTests.Documentation;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class StandaloneCheckTests
{
    [Fact]
    public async Task Check_binds_a_standalone_program_and_reports_source_errors_without_writing_outputs()
    {
        using var directory = new TemporaryDirectory(Path.Combine(DocumentationLinks.Repository, ".temp-data"));
        const string file = "Grüße helper.fpas";
        var source = directory.File(file);
        const string program = """
            program Helper;

            uses System.Console as Console;

            begin
              Console.WriteLine(42);
            end;

            end program;
            """;
        File.WriteAllText(source, program);

        var valid = await Check();
        Assert.True(valid.ExitCode == 0, valid.Output + valid.Error);
        Assert.Empty(valid.Error);
        using var success = JsonDocument.Parse(valid.Output);
        Assert.True(success.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal(source, success.RootElement.GetProperty("context").GetString());
        Assert.Empty(success.RootElement.GetProperty("projects").EnumerateArray());
        var snapshot = success.RootElement.GetProperty("snapshotId").GetString()!;
        Assert.NotEmpty(snapshot);

        var invalidProgram = program.Replace("42", "Missing", StringComparison.Ordinal);
        File.WriteAllText(source, invalidProgram);
        var invalid = await Check("--diagnostic-file", file);
        Assert.Equal(1, invalid.ExitCode);
        Assert.Empty(invalid.Error);
        using var failure = JsonDocument.Parse(invalid.Output);
        var diagnostic = Assert.Single(failure.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP3001", diagnostic.GetProperty("code").GetString());
        Assert.Equal(JsonValueKind.Null, diagnostic.GetProperty("project").ValueKind);
        var location = diagnostic.GetProperty("location");
        Assert.Equal(source, location.GetProperty("path").GetString());
        Assert.Equal(6, location.GetProperty("line").GetInt32());
        Assert.Equal(21, location.GetProperty("column").GetInt32());

        var changed = await Check("--expected-snapshot", snapshot);
        Assert.Equal(1, changed.ExitCode);
        using var stale = JsonDocument.Parse(changed.Output);
        Assert.Equal("inputs_changed", stale.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(source, Assert.Single(Directory.EnumerateFileSystemEntries(directory.Path)));
        Assert.Equal(invalidProgram, File.ReadAllText(source));

        Task<ProcessResult> Check(params string[] args)
            => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check", file, "--report", "json", .. args],
                workingDirectory: directory.Path);
    }
}
