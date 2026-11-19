// [Documentation](../../../docs/pascal/tools/refactoring.md#add-a-parameter)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Refactoring;
using System.Globalization;
using System.Text.Json;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class AddParameterTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Cli_regenerates_parameter_addition_and_preserves_existing_argument_order()
    {
        const string source = """
            program TraceCalls;
            uses System.Console as Console;
            var Trace: int := 0;
            function Step(N: int): int; begin Trace := Trace * 10 + N; return N; end function;
            function Sum(Value: int; Tail: int := 7): int; begin return Value + Tail; end function;
            function Run(): int;
            begin
              return Sum(Step(1), Step(2)) + Sum(Tail := Step(3), Value := Step(4)) + Sum(Step(5));
            end function;
            begin Console.WriteLine(Run()); Console.WriteLine(Trace); end;
            end program;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var project = directory.File("Game/Game.fpasproj");
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>");
        var selection = RenameFixture.Select(directory, source, "Sum(Step(5))");
        var path = directory.File(".temp-data/add.json");
        var preview = await RenameFixture.Cli(directory, "add-parameter", project, "--file", selection.File,
            "--line", selection.Line.ToString(CultureInfo.InvariantCulture),
            "--column", selection.Column.ToString(CultureInfo.InvariantCulture),
            "--name", "Limit", "--type", "int", "--argument", "Step(9)", "--plan", path);
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        Assert.Contains("Add parameter Limit: planned", preview.Output);
        Assert.Equal(source, File.ReadAllText(selection.File));
        var saved = File.ReadAllBytes(path);
        var plan = RefactorPlanJson.Read(saved);
        Assert.Equal("add-parameter", plan.Kind);
        Assert.Equal(new RefactorParameter("Limit", "int", "Step(9)"), plan.AddedParameter);
        Assert.Equal("Sum", Assert.Single(plan.Targets).Name);
        Assert.False(plan.ChangesPublicApi);
        Assert.Equal(5, plan.EditCount);

        var modified = plan with { AddedParameter = new("Limit", "int", "Step(8)") };
        File.WriteAllBytes(path, RefactorPlanJson.Serialize(modified with { Id = RefactorPlanJson.Identity(modified) }));
        CheckRejected(await Apply(), "invalid_plan", source);

        File.WriteAllBytes(path, saved);
        var applied = await Apply();
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        using (var report = JsonDocument.Parse(applied.Output))
        {
            Assert.True(report.RootElement.GetProperty("rechecked").GetBoolean());
            Assert.True(Assert.Single(report.RootElement.GetProperty("files").EnumerateArray()).GetProperty("written").GetBoolean());
        }

        var expected = source.Replace("Tail: int := 7", "Limit: int; Tail: int := 7")
            .Replace("Sum(Step(1), Step(2))", "Sum(Step(1), Tail := Step(2), Limit := Step(9))")
            .Replace("Sum(Tail := Step(3), Value := Step(4))", "Sum(Tail := Step(3), Value := Step(4), Limit := Step(9))")
            .Replace("Sum(Step(5))", "Sum(Step(5), Limit := Step(9))");
        Assert.Equal(expected, File.ReadAllText(selection.File));
        CheckRejected(await Apply(), "stale_plan", expected);

        var build = await sdk.Run(directory, "build", project, "-m:1", "--disable-build-servers");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await ProcessRunner.RunAsync([directory.File("Game/bin/Debug/net10.0/Game.dll")]);
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(new[] { "22", "12934959" }, run.Output.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries));

        Task<ProcessResult> Apply() => RenameFixture.Cli(directory, "apply", project, "--plan", path, "--no-restore", "--report", "json");

        void CheckRejected(ProcessResult result, string code, string unchangedSource)
        {
            Assert.Equal(1, result.ExitCode);
            using var report = JsonDocument.Parse(result.Output);
            Assert.Equal(code, report.RootElement.GetProperty("error").GetProperty("code").GetString());
            Assert.Equal(unchangedSource, File.ReadAllText(selection.File));
        }
    }

    [Theory]
    [InlineData("--name")]
    [InlineData("--type")]
    [InlineData("--argument")]
    public async Task Each_parameter_input_is_required_before_workspace_loading(string missing)
    {
        using var directory = new TemporaryDirectory();
        var arguments = new List<string>
        {
            "add-parameter", "Missing.fpasproj", "--file", "Missing.fpas", "--line", "1", "--column", "1",
            "--plan", "add.json", "--report", "json"
        };
        foreach (var (option, value) in new[] { ("--name", "Limit"), ("--type", "int"), ("--argument", "10") })
            if (option != missing) arguments.AddRange([option, value]);
        var result = await RenameFixture.Cli(directory, arguments.ToArray());
        Assert.Equal(2, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Contains(missing, report.RootElement.GetProperty("error").GetProperty("message").GetString());
        Assert.Empty(Directory.GetFileSystemEntries(directory.Path));
    }
}
