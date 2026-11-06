using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class PartialQueryTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private async Task Prepare(TemporaryDirectory directory, string source)
    {
        File.WriteAllText(directory.File("Game/Game.fpasproj"), Project());
        File.WriteAllText(directory.File("Game/Bridge.fpas"), source);
        var result = await sdk.Run(directory, "restore", "Game/Game.fpasproj"); Assert.True(result.ExitCode == 0, result.Output + result.Error);
    }

    private string Project(string? reference = null) => $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup>" +
        (reference is null ? "" : $"<ItemGroup><ProjectReference Include=\"{reference}\" /></ItemGroup>") + "</Project>";

    private static SourceSelection Select(string path, string text, string marker, bool last = false)
        => new(path, 1, (last ? text.LastIndexOf(marker, StringComparison.Ordinal) : text.IndexOf(marker, StringComparison.Ordinal)) + 1);

    [Fact]
    public async Task Queries_use_checked_regions_report_incomplete_searches_and_keep_rename_strict()
    {
        using var directory = sdk.CreateProject();
        const string source = "unit Sample; public function Target(): int; begin return 42; end function; " +
            "function Broken(): int; begin const Value: int := Target(); return Missing; end function; " +
            "function Healthy(): int; begin return Target(); end function; end unit;";
        await Prepare(directory, source); var path = directory.File("Game/Bridge.fpas");
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        workspace.Open(path, source, 1); var snapshot = await workspace.AnalyzeAsync();
        Assert.False(snapshot.Analysis.Success); var query = snapshot.Queries();
        var target = Select(path, source, "Target():"); var call = Select(path, source, "Target()", last: true);
        Assert.Equal("Target", query.Execute("definition", call).Definition!.Symbol.Name);
        Assert.Equal("callable", query.Execute("type", call).Type!.Type.Kind);
        var refs = query.Execute("refs", target); Assert.Single(refs.Refs!.References); Assert.False(refs.Coverage!.Complete); Assert.False(refs.Truncated);
        Assert.Contains(refs.Coverage.Projects.Single().Regions, region => region.State == "unavailable" && region.Reason == "binding_error");
        Assert.Empty(query.Find(new("Value", "exact")).Find!.Matches);
        Assert.Single(query.Calls("callees", Select(path, source, "Healthy():")).Callees!.Page!.Sites);
        Assert.Single(query.Calls("callers", target).Callers!.Page.Sites);
        Assert.Equal("analysis_unavailable", Assert.Throws<ContextFailure>(() => query.Execute("definition", Select(path, source, "Missing"))).Code);
        Assert.Equal("unavailable", query.Inspect(Select(path, source, "Broken():")).Inspect!.Source.Body.Availability);
        Assert.Equal("analysis_failed", Assert.Throws<ContextFailure>(() => snapshot.Rename(target, "Fetch")).Code);
        await workspace.VerifyAsync(snapshot);
        var repaired = source.Replace("Missing", "42", StringComparison.Ordinal); workspace.Change(path, 2, [new(repaired)]);
        var complete = await workspace.AnalyzeAsync(); Assert.True(complete.Analysis.Success); Assert.True(complete.Queries().Execute("refs", Select(path, repaired, "Target():")).Coverage!.Complete);
        Assert.Equal("content_modified", (await Assert.ThrowsAsync<ContextFailure>(() => workspace.VerifyAsync(snapshot))).Code);
        Assert.Equal(source, File.ReadAllText(path)); Assert.Empty(Directory.GetFiles(directory.File("Game"), "*.dll", SearchOption.AllDirectories));
    }

    [Theory]
    [InlineData("unit Broken; public function Incomplete(Value: ): int; begin return 1; end function; end unit;")]
    [InlineData("unit Broken; public function Incomplete(Value: MissingType): int; begin return 1; end function; end unit;")]
    public async Task Incomplete_declarations_keep_independent_source_units_queryable(string broken)
    {
        using var directory = sdk.CreateProject(); const string source = "unit Healthy; function Answer(): int; begin return 42; end function; end unit;";
        await Prepare(directory, source); var path = directory.File("Game/Bridge.fpas"); var invalid = directory.File("Game/Broken.fpas"); File.WriteAllText(invalid, broken);
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        var snapshot = await workspace.AnalyzeAsync(); var query = snapshot.Queries();
        Assert.Equal("Answer", query.Execute("definition", Select(path, source, "Answer():")).Definition!.Symbol.Name);
        Assert.Equal("analysis_unavailable", Assert.Throws<ContextFailure>(() => query.Execute("definition", Select(invalid, broken, "Incomplete"))).Code);
        var search = query.Find(new("Absent")); Assert.Empty(search.Find!.Matches); Assert.False(search.Coverage!.Complete);
        Assert.Contains(search.Coverage.Projects.Single().Regions, region => region.Span.Path == invalid && region.State == "unavailable");
        Assert.Equal("analysis_failed", Assert.Throws<ContextFailure>(() => snapshot.Rename(Select(path, source, "Answer():"), "Renamed")).Code);
    }

    [Fact]
    public async Task Broken_dependencies_block_their_consumers_while_an_independent_project_remains_available()
    {
        using var directory = sdk.CreateProject(); const string source = "unit Library; public function Value(): int; begin return Missing; end function; end unit;";
        await Prepare(directory, source);
        foreach (var name in new[] { "Consumer", "Independent" }) Directory.CreateDirectory(directory.File(name));
        File.WriteAllText(directory.File("Consumer/Consumer.fpasproj"), Project("../Game/Game.fpasproj"));
        const string consumer = "unit Consumer; uses Library as L; function Run(): int; begin return L.Value(); end function; end unit;";
        File.WriteAllText(directory.File("Consumer/Main.fpas"), consumer);
        const string independent = "unit Independent; function Answer(): int; begin return 42; end function; end unit;";
        File.WriteAllText(directory.File("Independent/Independent.fpasproj"), Project()); File.WriteAllText(directory.File("Independent/Main.fpas"), independent);
        File.WriteAllText(directory.File("Partial.slnx"), "<Solution><Configurations><BuildType Name=\"Debug\" /><Platform Name=\"Any CPU\" />" +
            "<ProjectType Name=\"FPAS\" Extension=\".fpasproj\" TypeId=\"{5D4B15AB-1024-4803-9015-4F069B13DE5A}\"><Platform Project=\"AnyCPU\" /></ProjectType></Configurations>" +
            "<Project Path=\"Game/Game.fpasproj\" /><Project Path=\"Consumer/Consumer.fpasproj\" /><Project Path=\"Independent/Independent.fpasproj\" /></Solution>");
        var restored = await sdk.Run(directory, "restore", "Partial.slnx"); Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        await using var workspace = new LiveWorkspace(new(directory.File("Partial.slnx"))); var snapshot = await workspace.AnalyzeAsync(); var query = snapshot.Queries();
        Assert.Equal("Answer", query.Execute("definition", Select(directory.File("Independent/Main.fpas"), independent, "Answer():")).Definition!.Symbol.Name);
        Assert.Equal("analysis_unavailable", Assert.Throws<ContextFailure>(() => query.Execute("definition", Select(directory.File("Consumer/Main.fpas"), consumer, "Run():"))).Code);
        Assert.Contains(query.Find(new("Answer")).Coverage!.Projects, project => project.Project == directory.File("Consumer/Consumer.fpasproj") && project.Availability == "unavailable");
        Assert.Empty(Directory.GetFiles(directory.File("Game"), "*.dll", SearchOption.AllDirectories));
    }
}
