// [Documentation](../../../docs/pascal/tools/diagnostics.md)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Diagnostics;

public sealed class DiagnosticWorkspaceTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Check_json_projects_the_argument_type_and_passing_mode()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            procedure Parse(Text: string);
            begin
              var Number: int := 0;
              discard int.TryParse(Text, Number);
            end procedure;
            end unit;
            """);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check",
            project.File("Game/Game.fpasproj"), "--report", "json"], timeLimit: TimeSpan.FromMinutes(2));
        Assert.Equal(1, result.ExitCode);
        Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var diagnostic = Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP3003", diagnostic.GetProperty("code").GetString());
        Assert.Equal(6, diagnostic.GetProperty("location").GetProperty("length").GetInt32());
        var argument = diagnostic.GetProperty("argument");
        Assert.Equal(2, argument.GetProperty("index").GetInt32());
        Assert.Equal("int", argument.GetProperty("type").GetString());
        Assert.Equal("value", argument.GetProperty("mode").GetString());
        var parameter = Assert.Single(argument.GetProperty("parameters").EnumerateArray());
        Assert.Equal("result", parameter.GetProperty("name").GetString());
        Assert.Equal("int", parameter.GetProperty("type").GetString());
        Assert.Equal("out", parameter.GetProperty("mode").GetString());
    }

    [Fact]
    public async Task Check_json_includes_a_missing_nested_variant_pattern()
    {
        using var project = sdk.CreateProject();
        var source = project.File("Game/Bridge.fpas");
        const string text = """
            unit Demo.Bridge;
            type Light = variant Off; On; end variant;
            type Switch = variant Disabled; Active(State: Light); end variant;
            function Read(Value: Switch): int;
            begin
              return case Value of
                when Switch.Disabled: 0;
                when Switch.Active(Light.Off): 1;
              end case;
            end function;
            end unit;
            """;
        File.WriteAllText(source, text);
        var check = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check",
            project.File("Game/Game.fpasproj"), "--report", "json"], timeLimit: TimeSpan.FromMinutes(3));
        Assert.Equal(1, check.ExitCode);
        Assert.Empty(check.Error);
        using var json = JsonDocument.Parse(check.Output);
        var diagnostic = Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP3025", diagnostic.GetProperty("code").GetString());
        Assert.Contains("Missing pattern: Switch.Active(Light.On).", diagnostic.GetProperty("message").GetString());
        var expectation = diagnostic.GetProperty("expectation");
        Assert.Equal("pattern", expectation.GetProperty("kind").GetString());
        Assert.Equal("Switch.Active(Light.On)", Assert.Single(expectation.GetProperty("expected").EnumerateArray()).GetString());
    }

    [Fact]
    public async Task Analysis_commands_preserve_the_same_compiler_diagnostic_payload()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge; public function Message(Value: string): string; begin const Count: integer := 1; return Value; end function; end unit;");
        var cli = TestEnvironment.Metadata("FpasCliPath");
        Task<ProcessResult> Run(params string[] arguments) => ProcessRunner.RunAsync([cli, .. arguments], timeLimit: TimeSpan.FromMinutes(3));
        var check = await Run("check", project.File("Game/Game.fpasproj"), "--report", "json");
        Assert.Equal(1, check.ExitCode); Assert.Equal("", check.Error);
        using var checkedJson = JsonDocument.Parse(check.Output);
        var diagnostic = Assert.Single(checkedJson.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("error", diagnostic.GetProperty("severity").GetString()); Assert.Equal("integer", diagnostic.GetProperty("expectation").GetProperty("found").GetString());
        Assert.Single(diagnostic.GetProperty("fixes").EnumerateArray());
        Assert.Equal(project.File("Game/Game.fpasproj"), diagnostic.GetProperty("project").GetString());
        var query = await Run("query", "type", project.File("Game/Game.fpasproj"), "--file", project.File("Game/Bridge.fpas"), "--line", "1", "--column", "1", "--no-restore");
        Assert.Equal(0, query.ExitCode); Assert.Equal("", query.Error);
        using var queriedJson = JsonDocument.Parse(query.Output);
        var queriedDiagnostic = Assert.Single(queriedJson.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal(diagnostic.GetProperty("expectation").GetRawText(), queriedDiagnostic.GetProperty("expectation").GetRawText());
        Assert.Equal(diagnostic.GetProperty("fixes").GetRawText(), queriedDiagnostic.GetProperty("fixes").GetRawText());
        var doc = await Run("doc", project.File("Game/Game.fpasproj"), "--name", "Message", "--report", "json", "--no-restore");
        Assert.Equal(0, doc.ExitCode); Assert.Empty(doc.Error);
        using var docJson = JsonDocument.Parse(doc.Output);
        var docDiagnostic = Assert.Single(docJson.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal(diagnostic.GetProperty("expectation").GetRawText(), docDiagnostic.GetProperty("expectation").GetRawText());
        Assert.Equal(diagnostic.GetProperty("fixes").GetRawText(), docDiagnostic.GetProperty("fixes").GetRawText());
        var text = await Run("check", project.File("Game/Game.fpasproj"), "--no-restore");
        Assert.Equal(1, text.ExitCode); Assert.Equal("", text.Output);
        Assert.Contains("|", text.Error); Assert.Contains("help:", text.Error); Assert.Contains("Use int", text.Error);
        var docText = await Run("doc", project.File("Game/Game.fpasproj"), "--name", "Message", "--no-restore");
        Assert.Equal(0, docText.ExitCode); Assert.Contains("Analysis is incomplete", docText.Output);
        Assert.Contains("Bridge.fpas(1,", docText.Error); Assert.Contains("help:", docText.Error); Assert.Contains("Use int", docText.Error);
    }

    [Fact]
    public async Task MSBuild_keeps_help_and_independent_routine_errors_in_the_same_report()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            public function Message(Value: string): string;
            begin
              const Count: int := 'text';
              return Value;
            end function;
            procedure Other();
            begin
              const Count: boolean := true;
            end procedure;
            end unit;
            """);
        var logger = await DiagnosticBuildLogger.Prepare(project);
        var build = await sdk.Run(project, "build", "Game/Game.fpasproj", "-m:1", "--disable-build-servers", logger);
        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains("FP3002", build.Output); Assert.Contains("FP3005", build.Output);
        Assert.Contains("help:", build.Output); Assert.Contains("fix: Use bool", build.Output);
        Assert.Contains("Bridge.fpas(4,", build.Output); Assert.Contains("Bridge.fpas(9,", build.Output);
        var errors = File.ReadAllLines(project.File("diagnostics.jsonl"));
        Assert.Equal(2, errors.Length);
        using var first = JsonDocument.Parse(errors[0]); using var second = JsonDocument.Parse(errors[1]);
        Assert.Equal("FP3002", first.RootElement.GetProperty("Code").GetString());
        Assert.Equal(4, first.RootElement.GetProperty("LineNumber").GetInt32());
        Assert.Equal(4, first.RootElement.GetProperty("EndLineNumber").GetInt32());
        Assert.True(first.RootElement.GetProperty("EndColumnNumber").GetInt32() > first.RootElement.GetProperty("ColumnNumber").GetInt32());
        Assert.Equal("FP3005", second.RootElement.GetProperty("Code").GetString());
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
    }
}
