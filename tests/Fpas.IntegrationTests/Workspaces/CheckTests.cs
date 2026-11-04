using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.IntegrationTests.Workspaces.Support;
using System.Security.Cryptography;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class CheckTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Check_returns_one_JSON_document_and_no_root_outputs()
    {
        using var project = sdk.CreateProject();
        var result = await Check(project, "Game/Game.fpasproj", "--diagnostics", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var body = json.RootElement;
        Assert.True(body.GetProperty("success").GetBoolean());
        Assert.True(body.GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.Equal("checked", body.GetProperty("projects")[0].GetProperty("state").GetString());
        Assert.NotEmpty(body.GetProperty("snapshotId").GetString()!);
        Assert.True(body.GetProperty("preparation").GetProperty("restorePerformed").GetBoolean());
        Assert.NotEmpty(body.GetProperty("preparation").GetProperty("dependencyOutputs").EnumerateArray());
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.False(File.Exists(project.File("Game/obj/Debug/net10.0/fpas.compile.json")));
    }

    [Fact]
    public async Task A_failed_check_preserves_existing_root_outputs_and_successful_build_fingerprint()
    {
        using var project = sdk.CreateProject();
        var projectPath = project.File("Game/Game.fpasproj"); var projectXml = XDocument.Load(projectPath);
        projectXml.Root!.Descendants("OutputType").Single().Value = "Exe"; projectXml.Save(projectPath);
        File.WriteAllText(project.File("Game/Bridge.fpas"), "program Sample; begin end; end program;");
        var built = await sdk.Run(project, "build", "Game/Game.fpasproj", "-p:GenerateDocumentationFile=true", "-m:1", "--disable-build-servers");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var outputs = Directory.GetFiles(project.File("Game/bin"), "*", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(project.File("Game/obj"), "fpas.compile.json*", SearchOption.AllDirectories))
            .ToDictionary(path => path, Hash);
        Assert.Contains(outputs.Keys, path => path.EndsWith("Game.pdb", StringComparison.Ordinal));
        Assert.Contains(outputs.Keys, path => path.EndsWith("Game.xml", StringComparison.Ordinal));
        Assert.Contains(outputs.Keys, path => path.EndsWith("Game.runtimeconfig.json", StringComparison.Ordinal));
        File.WriteAllText(project.File("Game/Bridge.fpas"), "program Sample; begin const Value: int := Missing; end; end program;");
        var result = await Check(project, "Game/Game.fpasproj", "--no-restore", "--diagnostics", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid", json.RootElement.GetProperty("projects")[0].GetProperty("state").GetString());
        Assert.Contains(json.RootElement.GetProperty("diagnostics").EnumerateArray(), diagnostic => diagnostic.GetProperty("code").GetString() == "FP3001");
        foreach (var (path, hash) in outputs) Assert.Equal(hash, Hash(path));
    }

    [Fact]
    public async Task Check_collects_independent_diagnostics_and_marks_failed_dependents_blocked()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge; public function Answer(): int; begin return Missing; end function; end unit;");
        var solutionPath = project.File("Game.slnx"); var solution = XDocument.Load(solutionPath);
        foreach (var name in new[] { "Independent", "Dependent" })
        {
            Directory.CreateDirectory(project.File(name));
            File.WriteAllText(project.File(name + "/Main.fpas"), "program " + name + "; begin end; end program;");
            File.WriteAllText(project.File(name + "/" + name + ".fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup>" +
                (name == "Dependent" ? "<ItemGroup><ProjectReference Include=\"../Game/Game.fpasproj\" /></ItemGroup>" : "") + "</Project>");
            solution.Root!.Add(new XElement("Project", new XAttribute("Path", name + "/" + name + ".fpasproj")));
        }
        solution.Save(solutionPath);
        var result = await Check(project, "Game.slnx", "--diagnostics", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var projects = json.RootElement.GetProperty("projects").EnumerateArray().ToArray();
        Assert.Equal(3, projects.Length);
        Assert.Contains(projects, item => item.GetProperty("project").GetString() == project.File("Independent/Independent.fpasproj") && item.GetProperty("state").GetString() == "checked");
        Assert.Contains(projects, item => item.GetProperty("project").GetString() == project.File("Dependent/Dependent.fpasproj") && item.GetProperty("state").GetString() == "blocked");
        Assert.False(json.RootElement.GetProperty("coverage").GetProperty("complete").GetBoolean());
    }

    [Fact]
    public async Task Documentation_warning_policy_is_used_and_default_diagnostics_go_to_stderr()
    {
        using var project = sdk.CreateProject();
        var path = project.File("Game/Game.fpasproj"); var xml = XDocument.Load(path);
        xml.Root!.Add(new XElement("PropertyGroup", new XElement("GenerateDocumentationFile", "true"), new XElement("TreatWarningsAsErrors", "true")));
        xml.Save(path);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            // Read a value.
            // ## Parameters
            // - `Wrong`: Not a parameter.
            public function Read(Value: int): int;
            begin return Value; end function;
            end unit;
            """);
        var result = await Check(project, "Game/Game.fpasproj");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Output);
        Assert.Contains("error FP6001:", result.Error);
        Assert.Contains("Checked 1/1", result.Error);
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.xml")));
    }

    [Theory]
    [InlineData("--unknown", "bad")]
    [InlineData("--timeout", "0")]
    [InlineData("--timeout", "3601")]
    [InlineData("--timeout", "abc")]
    public async Task Invalid_check_arguments_still_return_structured_JSON(string option, string value)
    {
        using var project = sdk.CreateProject();
        var result = await Check(project, "Game/Game.fpasproj", option, value, "--diagnostics", "json");
        Assert.Equal(2, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Fact]
    public async Task Timeout_is_a_single_structured_failure_and_joins_preparation()
    {
        using var project = sdk.CreateProject();
        using var waiting = await WaitingBuild.Prepare(project.File("Game/Game.fpasproj"));
        var result = await Check(project, "Game/Game.fpasproj", "--timeout", "1", "--diagnostics", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("timeout", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    private Task<ProcessResult> Check(TemporaryDirectory project, params string[] arguments)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check", .. arguments], workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(3));
    private static string Hash(string path) => Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path)));
}
