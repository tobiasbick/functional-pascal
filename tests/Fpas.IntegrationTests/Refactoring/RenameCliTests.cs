// [Documentation](../../../docs/pascal/tools/refactoring.md)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Refactoring;
using System.Text.Json;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class RenameCliTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Source = "unit Demo.Bridge; public function Read(Value: int): int; begin return Value; end function; public function Run(): int; begin return Read(42); end function; end unit;";

    [Fact]
    public async Task A_found_overload_can_be_inspected_referenced_and_renamed_without_source_coordinates()
    {
        const string source = """
            unit Demo.Bridge;
            function Transform(Value: int): int; begin return Value + 1; end function;
            function Transform(Value: string): int; begin return Value.Length; end function;
            public function Run(): int;
            begin return Transform(40) + Transform('text'); end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        const string context = "Game/Game.fpasproj";
        const string sourceFile = "Game/Bridge.fpas";
        var found = await Query("find", ["--name", "Transform", "--match", "exact"]);
        Assert.True(found.ExitCode == 0, found.Output + found.Error);
        using var find = JsonDocument.Parse(found.Output);
        var matches = find.RootElement.GetProperty("result").GetProperty("find").GetProperty("matches").EnumerateArray().ToArray();
        Assert.Equal(2, matches.Length);
        var target = Assert.Single(matches, match => match.GetProperty("signature").GetString()!.Contains("Value: int", StringComparison.Ordinal));
        var symbol = target.GetProperty("symbol").GetString()!;
        var project = target.GetProperty("project").GetString()!;
        var snapshot = find.RootElement.GetProperty("snapshotId").GetString()!;
        string[] selection = ["--existing-inputs", "--symbol", symbol, "--project", project, "--expected-snapshot", snapshot];
        using (var inspected = await ReadQuery("inspect", selection))
        {
            var text = inspected.RootElement.GetProperty("result").GetProperty("inspect").GetProperty("source").GetProperty("declaration").GetProperty("text").GetString();
            Assert.Contains("Transform(Value: int)", text);
        }
        using (var referenced = await ReadQuery("refs", selection))
            Assert.Equal(1, referenced.RootElement.GetProperty("result").GetProperty("refs").GetProperty("returnedCount").GetInt32());
        var batch = await Query("batch", ["--existing-inputs", "--expected-snapshot", snapshot],
            JsonSerializer.Serialize(new[] { new { operation = "refs", symbol, project } }));
        Assert.True(batch.ExitCode == 0, batch.Output + batch.Error);
        using (var report = JsonDocument.Parse(batch.Output))
            Assert.Equal(1, report.RootElement.GetProperty("results")[0].GetProperty("result").GetProperty("refs").GetProperty("returnedCount").GetInt32());

        var unpinned = await Query("refs", ["--symbol", symbol, "--project", project]);
        Assert.Equal(2, unpinned.ExitCode);
        var badId = await Query("refs", ["--existing-inputs", "--symbol", "unknown", "--project", project, "--expected-snapshot", snapshot]);
        Assert.Equal(2, badId.ExitCode);

        const string plan = "rename-symbol.json";
        var prepared = await RenameFixture.Cli(directory, ["rename", context, .. selection, "--plan", plan, "--new-name", "Map", "--report", "json"]);
        Assert.True(prepared.ExitCode == 0, prepared.Output + prepared.Error);
        Assert.Equal(source, File.ReadAllText(directory.File(sourceFile)));
        var applied = await RenameFixture.Cli(directory, "apply", context, "--existing-inputs", "--plan", plan, "--report", "json");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        var changed = File.ReadAllText(directory.File(sourceFile));
        Assert.Contains("function Map(Value: int)", changed);
        Assert.Contains("return Map(40) + Transform('text')", changed);
        var stale = await Query("refs", selection);
        Assert.Equal(1, stale.ExitCode);
        using var rejected = JsonDocument.Parse(stale.Output);
        Assert.Equal("inputs_changed", rejected.RootElement.GetProperty("error").GetProperty("code").GetString());

        Task<ProcessResult> Query(string operation, string[] arguments, string? input = null)
            => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", operation, context, .. arguments],
                input, directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        async Task<JsonDocument> ReadQuery(string operation, string[] arguments)
        {
            var result = await Query(operation, arguments);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            return JsonDocument.Parse(result.Output);
        }
    }

    /// <summary>Checks that invalid plans leave source files unchanged and that a valid plan applies once.</summary>
    [Fact]
    public async Task Planning_is_read_only_apply_rechecks_and_stale_or_modified_plans_never_write()
    {
        using var directory = RenameFixture.Create(sdk, Source); var selection = RenameFixture.Select(directory, Source, "Read(Value");
        var arguments = new[] { "rename", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "1", "--column", selection.Column.ToString(System.Globalization.CultureInfo.InvariantCulture),
            "--new-name", "Fetch", "--plan", ".temp-data/rename.json", "--report", "json" };
        var planned = await RenameFixture.Cli(directory, arguments); Assert.True(planned.ExitCode == 0, planned.Output + planned.Error); Assert.Empty(planned.Error);
        var path = directory.File(".temp-data/rename.json"); var original = File.ReadAllBytes(path);
        var plan = RefactorPlanJson.Read(original); Assert.Equal(2, plan.EditCount); Assert.True(plan.ChangesPublicApi);
        Assert.Equal(Source, File.ReadAllText(directory.File("Game/Bridge.fpas"))); Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
        var overwrite = await RenameFixture.Cli(directory, arguments); Assert.Equal(1, overwrite.ExitCode); Assert.Equal(original, File.ReadAllBytes(path));
        var mismatch = await RenameFixture.Cli(directory, "apply", "Missing.fpasproj", "--plan", path, "--report", "json");
        Assert.Equal(1, mismatch.ExitCode); Assert.False(File.Exists(directory.File("Missing.fpasproj")));
        using (var json = JsonDocument.Parse(mismatch.Output)) Assert.Equal("plan_context_mismatch", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        using (var stream = File.OpenWrite(path)) stream.SetLength(RefactorPlanJson.MaximumBytes + 1);
        var oversized = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--report", "json");
        Assert.Equal(1, oversized.ExitCode);
        using (var json = JsonDocument.Parse(oversized.Output)) Assert.Equal("invalid_plan", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(Source, File.ReadAllText(directory.File("Game/Bridge.fpas")));
        File.WriteAllText(path, System.Text.Encoding.UTF8.GetString(original).Replace("Fetch", "Broken", StringComparison.Ordinal));
        var modified = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--report", "json");
        Assert.Equal(1, modified.ExitCode); using (var json = JsonDocument.Parse(modified.Output)) Assert.Equal("invalid_plan", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        File.WriteAllBytes(path, original);
        var applied = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore", "--report", "json");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error); Assert.Empty(applied.Error);
        using (var json = JsonDocument.Parse(applied.Output))
        { Assert.True(json.RootElement.GetProperty("rechecked").GetBoolean()); Assert.All(json.RootElement.GetProperty("files").EnumerateArray(), file => Assert.True(file.GetProperty("written").GetBoolean())); }
        Assert.Equal(Source.Replace("Read", "Fetch", StringComparison.Ordinal), File.ReadAllText(directory.File("Game/Bridge.fpas")));
        var stale = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore", "--report", "json");
        Assert.Equal(1, stale.ExitCode);
        using (var json = JsonDocument.Parse(stale.Output))
        {
            var error = json.RootElement.GetProperty("error");
            Assert.Equal("stale_plan", error.GetProperty("code").GetString());
            Assert.Equal("recreate_plan", error.GetProperty("nextAction").GetString());
        }
    }

    [Fact]
    public async Task Positions_require_the_expected_snapshot_before_querying_or_writing_a_plan()
    {
        using var directory = RenameFixture.Create(sdk, Source);
        var cli = TestEnvironment.Metadata("FpasCliPath");
        const string context = "Game/Game.fpasproj";
        const string file = "Game/Bridge.fpas";
        var find = await ProcessRunner.RunAsync([cli, "query", "find", context, "--name", "Read", "--match", "exact"],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(find.ExitCode == 0, find.Output + find.Error);
        using var found = JsonDocument.Parse(find.Output);
        var snapshot = found.RootElement.GetProperty("snapshotId").GetString()!;
        var selection = RenameFixture.Select(directory, Source, "Read(Value");
        var position = new[] { context, "--file", file, "--line", "1", "--column",
            selection.Column.ToString(System.Globalization.CultureInfo.InvariantCulture), "--expected-snapshot", snapshot, "--existing-inputs" };
        var reply = await ProcessRunner.RunAsync([cli, "query", "definition", .. position],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(reply.ExitCode == 0, reply.Output + reply.Error);
        using var accepted = JsonDocument.Parse(reply.Output);
        Assert.Equal(snapshot, accepted.RootElement.GetProperty("snapshotId").GetString());

        var changed = "\n" + Source;
        File.WriteAllText(directory.File(file), changed);
        var query = await ProcessRunner.RunAsync([cli, "query", "definition", .. position],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        var expected = Rejected(query);
        var plan = await RenameFixture.Cli(directory, ["rename", .. position, "--new-name", "Fetch",
            "--plan", ".temp-data/guarded.json", "--report", "json"]);
        Assert.Equal(expected, Rejected(plan));
        Assert.False(File.Exists(directory.File(".temp-data/guarded.json")));
        Assert.Equal(changed, File.ReadAllText(directory.File(file)));

        string? Rejected(ProcessResult result)
        {
            Assert.Equal(1, result.ExitCode);
            using var json = JsonDocument.Parse(result.Output);
            var error = json.RootElement.GetProperty("error");
            Assert.Equal("inputs_changed", error.GetProperty("code").GetString());
            Assert.Equal("refresh_snapshot", error.GetProperty("nextAction").GetString());
            Assert.Equal(snapshot, error.GetProperty("expectedSnapshotId").GetString());
            return error.GetProperty("actualSnapshotId").GetString();
        }
    }

    [Theory]
    [InlineData("bad.name")]
    [InlineData("@end")]
    [InlineData("_")]
    [InlineData("1Bad")]
    public async Task Invalid_names_fail_before_preparation(string name)
    {
        using var directory = new Fpas.Tests.Support.TemporaryDirectory();
        var result = await RenameFixture.Cli(directory, "rename", "Missing.fpasproj", "--file", "Missing.fpas", "--line", "1", "--column", "1", "--new-name", name, "--plan", "rename.json", "--report", "json");
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error); Assert.Empty(Directory.GetFileSystemEntries(directory.Path));
        using var json = JsonDocument.Parse(result.Output); Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }
}
