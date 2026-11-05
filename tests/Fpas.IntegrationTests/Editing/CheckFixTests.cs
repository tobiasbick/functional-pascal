using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Editing;

public sealed class CheckFixTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Source = "unit Demo.Bridge; public function Message(Value: integer): int; begin return Value; end function; end unit;\r\n";
    private static Task<ProcessResult> Run(TemporaryDirectory directory, params string[] options)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check", "Game/Game.fpasproj", "--diagnostics", "json", .. options],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(3));

    [Fact]
    public async Task Preview_apply_recheck_and_idempotence_use_real_compiler_edits_and_preserve_encoding()
    {
        using var project = sdk.CreateProject(); var path = project.File("Game/Bridge.fpas");
        var bytes = Encoding.UTF8.GetBytes("\uFEFF" + Source); File.WriteAllBytes(path, bytes);
        var preview = await Run(project, "--fix", "--dry-run"); Assert.Equal(1, preview.ExitCode); Assert.Empty(preview.Error);
        using (var json = JsonDocument.Parse(preview.Output))
        {
            var fixes = json.RootElement.GetProperty("fixes"); Assert.Equal("preview", fixes.GetProperty("mode").GetString());
            Assert.Equal(1, fixes.GetProperty("editCount").GetInt32()); Assert.False(Assert.Single(fixes.GetProperty("files").EnumerateArray()).GetProperty("written").GetBoolean());
            Assert.NotEmpty(fixes.GetProperty("inputSnapshotId").GetString()!);
        }
        Assert.Equal(bytes, File.ReadAllBytes(path));
        var location = await sdk.Run(project, "msbuild", "Game/Game.fpasproj", "-nologo", "-getProperty:NuGetPackageRoot"); Assert.Equal(0, location.ExitCode);
        var packagedCli = Path.Combine(location.Output.Trim(), "fpas.net.sdk", sdk.Version.ToLowerInvariant(), "tools", "compiler", "Fpas.Cli.dll");
        var applied = await ProcessRunner.RunAsync([packagedCli, "check", "Game/Game.fpasproj", "--fix", "--no-restore", "--diagnostics", "json"],
            workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(3));
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error); Assert.Empty(applied.Error);
        using (var json = JsonDocument.Parse(applied.Output))
        {
            Assert.True(json.RootElement.GetProperty("success").GetBoolean()); Assert.Empty(json.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.Equal("applied", json.RootElement.GetProperty("fixes").GetProperty("state").GetString());
            Assert.True(json.RootElement.GetProperty("fixes").GetProperty("rechecked").GetBoolean());
            Assert.True(Assert.Single(json.RootElement.GetProperty("fixes").GetProperty("files").EnumerateArray()).GetProperty("written").GetBoolean());
        }
        Assert.Equal(Encoding.UTF8.GetBytes("\uFEFF" + Source.Replace("integer", "int", StringComparison.Ordinal)), File.ReadAllBytes(path));
        var stamp = File.GetLastWriteTimeUtc(path); var again = await Run(project, "--fix", "--no-restore"); Assert.Equal(0, again.ExitCode);
        Assert.Equal(stamp, File.GetLastWriteTimeUtc(path)); Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.Empty(Directory.GetFiles(project.File("Game"), ".fpas-edit-*"));
    }

    [Fact]
    public async Task Remaining_errors_and_review_only_proposals_are_not_reported_as_success()
    {
        using var project = sdk.CreateProject(); var path = project.File("Game/Bridge.fpas");
        File.WriteAllText(path, Source.Replace("return Value", "return Missing", StringComparison.Ordinal));
        var applied = await Run(project, "--fix"); Assert.Equal(1, applied.ExitCode);
        using (var json = JsonDocument.Parse(applied.Output))
        {
            Assert.Equal("applied", json.RootElement.GetProperty("fixes").GetProperty("state").GetString());
            Assert.Contains(json.RootElement.GetProperty("diagnostics").EnumerateArray(), diagnostic => diagnostic.GetProperty("code").GetString() == "FP3001");
        }
        Assert.DoesNotContain("integer", File.ReadAllText(path)); Assert.Contains("Missing", File.ReadAllText(path));
        var review = "unit Demo.Bridge; procedure P(); begin if true then discard 1; elseif false then discard 2; end if; end procedure; end unit;";
        File.WriteAllText(path, review); var refused = await Run(project, "--fix", "--no-restore"); Assert.Equal(1, refused.ExitCode);
        using var report = JsonDocument.Parse(refused.Output);
        Assert.True(report.RootElement.GetProperty("fixes").GetProperty("reviewCount").GetInt32() > 0);
        Assert.Equal(review, File.ReadAllText(path));
    }

    [Fact]
    public async Task A_solution_batch_fixes_independent_projects_and_preserves_existing_root_outputs()
    {
        using var project = sdk.CreateProject();
        var built = await sdk.Run(project, "build", "Game/Game.fpasproj", "-m:1", "--disable-build-servers"); Assert.Equal(0, built.ExitCode);
        var outputs = Directory.GetFiles(project.File("Game/bin"), "*", SearchOption.AllDirectories)
            .Concat(Directory.GetFiles(project.File("Game/obj"), "fpas.compile.json*", SearchOption.AllDirectories))
            .ToDictionary(path => path, File.ReadAllBytes);
        File.WriteAllText(project.File("Game/Bridge.fpas"), Source);
        Directory.CreateDirectory(project.File("Other"));
        File.WriteAllText(project.File("Other/Other.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(project.File("Other/Other.fpas"), "unit Other; public function Read(Value: integer): int; begin return Value; end function; end unit;");
        var solution = XDocument.Load(project.File("Game.slnx")); solution.Root!.Add(new XElement("Project", new XAttribute("Path", "Other/Other.fpasproj"))); solution.Save(project.File("Game.slnx"));
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check", "Game.slnx", "--fix", "--diagnostics", "json"],
            workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(3));
        Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal(2, report.RootElement.GetProperty("fixes").GetProperty("files").GetArrayLength());
        Assert.All(report.RootElement.GetProperty("fixes").GetProperty("files").EnumerateArray(), file => Assert.True(file.GetProperty("written").GetBoolean()));
        foreach (var (path, bytes) in outputs) Assert.Equal(bytes, File.ReadAllBytes(path));
    }

    [Fact]
    public async Task Read_only_sources_and_failed_dependency_preparation_never_apply_a_partial_batch()
    {
        using var project = sdk.CreateProject(); var path = project.File("Game/Bridge.fpas"); File.WriteAllText(path, Source);
        var attributes = File.GetAttributes(path);
        try
        {
            File.SetAttributes(path, attributes | FileAttributes.ReadOnly);
            var result = await Run(project, "--fix"); Assert.Equal(1, result.ExitCode); Assert.Empty(result.Error);
            using var json = JsonDocument.Parse(result.Output);
            Assert.False(Assert.Single(json.RootElement.GetProperty("fixes").GetProperty("files").EnumerateArray()).GetProperty("written").GetBoolean());
            Assert.Equal("failed", json.RootElement.GetProperty("fixes").GetProperty("state").GetString());
        }
        finally { File.SetAttributes(path, attributes); }
        Assert.Equal(Source, File.ReadAllText(path));
        File.WriteAllText(project.File("Shared/Broken.cs"), "not C# source");
        var blocked = await Run(project, "--fix", "--no-restore"); Assert.Equal(1, blocked.ExitCode);
        using var report = JsonDocument.Parse(blocked.Output);
        Assert.Equal("fix_incomplete", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(Source, File.ReadAllText(path)); Assert.Empty(Directory.GetFiles(project.File("Game"), ".fpas-edit-*"));
    }

    [Theory]
    [InlineData("Unit Demo.Bridge; public function Message(Value: int): int; begin return Value; end function; end unit;", "unit Demo.Bridge; public function Message(Value: int): int; begin return Value; end function; end unit;")]
    [InlineData("unit Demo.Bridge; public function Message(Value: int): int; begin return Value end function; end unit;", "unit Demo.Bridge; public function Message(Value: int): int; begin return Value ;end function; end unit;")]
    public async Task Parser_corrections_are_applied_and_checked_again(string before, string after)
    {
        using var project = sdk.CreateProject(); var path = project.File("Game/Bridge.fpas"); File.WriteAllText(path, before);
        var result = await Run(project, "--fix"); Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Empty(result.Error);
        Assert.Equal(after, File.ReadAllText(path));
    }

    [Fact]
    public async Task Dry_run_without_fix_fails_before_project_preparation()
    {
        using var project = new TemporaryDirectory(); var result = await Run(project, "--dry-run");
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error); Assert.Empty(Directory.GetFileSystemEntries(project.Path));
        using var json = JsonDocument.Parse(result.Output); Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }
}
