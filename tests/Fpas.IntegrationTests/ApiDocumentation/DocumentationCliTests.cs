using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Build.Support;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class DocumentationCliTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task The_command_looks_up_unused_framework_APIs_and_reports_source_and_XML_documentation()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var project = directory.File("Game/Game.fpasproj");
        var native = await DocumentationFixture.Cli(directory, project, "--name", "System.Console.WriteLine", "--report", "json");
        Assert.True(native.ExitCode == 0, native.Output + native.Error); Assert.Empty(native.Error);
        using (var report = JsonDocument.Parse(native.Output))
        {
            Assert.Equal("ok", report.RootElement.GetProperty("status").GetString());
            var entries = report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray().ToArray();
            Assert.True(entries.Length > 5);
            Assert.All(entries, entry => { Assert.Equal("metadata", entry.GetProperty("origin").GetString()); Assert.Equal("available", entry.GetProperty("documentation").GetProperty("availability").GetString()); });
            Assert.Contains(entries, entry => entry.GetProperty("documentationId").GetString() == "M:System.Console.WriteLine(System.Int32)");
        }
        var source = await DocumentationFixture.Cli(directory, project, "--name", "Message", "--no-restore");
        Assert.True(source.ExitCode == 0, source.Output + source.Error); Assert.Empty(source.Error); Assert.Contains("Echoes the provided source value.", source.Output);
        Assert.Contains("\n## Parameters\n", source.Output.Replace("\r\n", "\n", StringComparison.Ordinal));
        var missing = await DocumentationFixture.Cli(directory, project, "--name", "Absent", "--report", "json", "--no-restore");
        Assert.Equal(0, missing.ExitCode); using var notFound = JsonDocument.Parse(missing.Output); Assert.Equal("not_found", notFound.RootElement.GetProperty("status").GetString());
        Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    [Theory]
    [InlineData("--name", "")]
    [InlineData("--match", "regex")]
    [InlineData("--limit", "0")]
    [InlineData("--offset", "-1")]
    [InlineData("--timeout", "0")]
    [InlineData("--unknown", "value")]
    public async Task Argument_failures_are_one_JSON_document_before_preparation(string option, string value)
    {
        using var directory = new TemporaryDirectory();
        var args = new List<string> { "Missing.fpasproj", "--report", "json" };
        if (option != "--name") args.AddRange(["--name", "X"]);
        args.AddRange([option, value]);
        var result = await DocumentationFixture.Cli(directory, args.ToArray());
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output); Assert.Equal("invalid_argument", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Empty(Directory.GetFileSystemEntries(directory.Path));
    }

    [Fact]
    public async Task Deadline_returns_one_structured_failure_after_owned_preparation_stops()
    {
        using var directory = DocumentationFixture.Create(sdk);
        using var waiting = await WaitingBuild.Prepare(directory.File("Game/Game.fpasproj"));
        var result = await DocumentationFixture.Cli(directory, directory.File("Game/Game.fpasproj"), "--name", "Message", "--timeout", "1", "--report", "json");
        Assert.Equal(1, result.ExitCode); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("timeout", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(report.RootElement.TryGetProperty("result", out _));
    }

    [Fact]
    public async Task Incomplete_source_never_claims_complete_absence_in_documentation_lookup()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Game/Bridge.fpas"), "unit Broken; public function F(): int; begin return Missing; end function; end unit;");
        var result = await DocumentationFixture.Cli(directory, directory.File("Game/Game.fpasproj"), "--name", "Absent", "--report", "json");
        Assert.Equal(0, result.ExitCode); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output); Assert.Equal("ok", report.RootElement.GetProperty("status").GetString());
        Assert.Empty(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray());
        Assert.False(report.RootElement.GetProperty("result").GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.Equal("partial", report.RootElement.GetProperty("coverage").GetProperty("semantics")[0].GetProperty("availability").GetString());
    }
}
