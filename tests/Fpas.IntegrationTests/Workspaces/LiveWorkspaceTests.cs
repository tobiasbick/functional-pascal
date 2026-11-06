using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class LiveWorkspaceTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private async Task Prepare(TemporaryDirectory directory, string path = "Game/Game.fpasproj")
    {
        var build = await sdk.Run(directory, "build", path, "--disable-build-servers"); Assert.True(build.ExitCode == 0, build.Output + build.Error);
    }

    private static SourceSelection Select(string path, string text, string marker)
    {
        var offset = text.IndexOf(marker, StringComparison.Ordinal); Assert.True(offset >= 0);
        var lines = text[..offset].Split('\n'); return new(path, lines.Length, lines[^1].Length + 1);
    }

    [Fact]
    public async Task Unsaved_dependency_signatures_queries_and_rename_use_private_images_without_writes_or_builds()
    {
        using var directory = sdk.CreateProject(); var libraryPath = directory.File("Game/Bridge.fpas");
        const string library = "unit Demo.Bridge; public function Read(): int; begin return 42; end function; end unit;";
        const string originalConsumer = "unit Consumer; uses Demo.Bridge as B; public function Run(): int; begin return B.Read(); end function; end unit;";
        File.WriteAllText(libraryPath, library); Directory.CreateDirectory(directory.File("Consumer"));
        var consumerPath = directory.File("Consumer/Main.fpas"); File.WriteAllText(consumerPath, originalConsumer);
        File.WriteAllText(directory.File("Consumer/Consumer.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup><ItemGroup><ProjectReference Include=\"../Game/Game.fpasproj\" /></ItemGroup></Project>");
        await Prepare(directory, "Consumer/Consumer.fpasproj");
        var projectPath = directory.File("Game/Game.fpasproj"); var project = XDocument.Load(projectPath);
        project.Root!.Add(new XElement("Target", new XAttribute("Name", "ObserveGenerator"), new XAttribute("BeforeTargets", "FpasPrepareCompilation"),
            new XElement("Touch", new XAttribute("Files", "generator.marker"), new XAttribute("AlwaysCreate", "true")))); project.Save(projectPath);
        var outputs = Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories).Where(path => path.EndsWith(".fpas", StringComparison.Ordinal) ||
            path.Contains(Path.DirectorySeparatorChar + "bin" + Path.DirectorySeparatorChar, StringComparison.Ordinal)).ToDictionary(path => path, path => (Bytes: File.ReadAllBytes(path), Stamp: File.GetLastWriteTimeUtc(path)));
        await using var workspace = new LiveWorkspace(new(directory.File("Consumer/Consumer.fpasproj")));
        var revisedLibrary = library.Replace("Read(): int", "Read(Value: int): int", StringComparison.Ordinal).Replace("return 42", "return Value", StringComparison.Ordinal);
        var revisedConsumer = originalConsumer.Replace("B.Read()", "B.Read(Value := 42)", StringComparison.Ordinal);
        workspace.Open(libraryPath, revisedLibrary, 7); workspace.Open(consumerPath, revisedConsumer, 11);
        var snapshot = await workspace.AnalyzeAsync();
        Assert.True(snapshot.Analysis.Success, string.Join(';', snapshot.Analysis.Context.Issues.Select(issue => issue.Message)));
        var definition = snapshot.Queries().Execute("definition", Select(consumerPath, revisedConsumer, "Read(Value"));
        Assert.Equal(projectPath, definition.Definition!.Project); Assert.Equal("Read", definition.Definition.Symbol.Name);
        var rename = snapshot.Rename(Select(consumerPath, revisedConsumer, "Read(Value"), "Fetch");
        Assert.Equal(2, rename.Edits.Documents.Count); Assert.Equal([7, 11], rename.Edits.Documents.Select(document => document.Original.Version).Order().ToArray());
        Assert.Contains(rename.Edits.Documents, document => document.NewText.Contains("Fetch(Value: int)", StringComparison.Ordinal));
        await workspace.VerifyAsync(snapshot);
        workspace.Change(libraryPath, 8, [new(revisedLibrary.Replace("Value: int", "Value: int; Other: int", StringComparison.Ordinal))]);
        var invalidated = await workspace.AnalyzeAsync(); Assert.False(invalidated.Analysis.Success);
        Assert.Equal("invalid", invalidated.Analysis.Projects.Single(project => project.Project == directory.File("Consumer/Consumer.fpasproj")).State);
        workspace.Change(libraryPath, 9, [new(revisedLibrary)]); Assert.True((await workspace.AnalyzeAsync()).Analysis.Success);
        Assert.False(File.Exists(directory.File("Game/generator.marker")));
        foreach (var (path, value) in outputs) { Assert.Equal(value.Bytes, File.ReadAllBytes(path)); Assert.Equal(value.Stamp, File.GetLastWriteTimeUtc(path)); }
    }

    [Fact]
    public async Task Save_retains_open_authority_close_reloads_disk_and_old_results_are_rejected()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Bridge.fpas");
        const string source = "unit Demo.Bridge; public function Read(): int; begin return 42; end function; end unit;";
        File.WriteAllText(path, source); await Prepare(directory);
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        workspace.Open(path, source.Replace("Read", "Unsaved", StringComparison.Ordinal), 1);
        var first = await workspace.AnalyzeAsync(); Assert.True(first.Analysis.Success);
        var next = directory.File("Loose.fpas"); workspace.Open(next, "unit Loose; end unit;", 1);
        File.WriteAllText(path, source.Replace("Read", "OnDisk", StringComparison.Ordinal)); workspace.Save(path);
        var saved = await workspace.AnalyzeAsync(); Assert.Contains("Unsaved", saved.Documents.Document(path).Text);
        Assert.Single(saved.Unowned); Assert.Empty(saved.Unowned[0].Diagnostics);
        await workspace.VerifyAsync(saved);
        Assert.Equal("content_modified", (await Assert.ThrowsAsync<ContextFailure>(() => workspace.VerifyAsync(first))).Code);
        workspace.Close(path); workspace.Close(next); Assert.Null(workspace.Current);
        var closed = await workspace.AnalyzeAsync(); Assert.Contains("OnDisk", closed.Documents.Document(path).Text); Assert.Null(closed.Documents.Document(path).Version);
        Assert.Empty(closed.Unowned); Assert.DoesNotContain(closed.Documents.Documents, document => document.Path == next);
    }

    [Fact]
    public async Task Unchanged_semantics_reuse_project_facts_but_versions_and_revisions_still_invalidate_results()
    {
        using var directory = sdk.CreateProject(); await Prepare(directory);
        var path = directory.File("Game/Bridge.fpas"); var text = File.ReadAllText(path);
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        var disk = await workspace.AnalyzeAsync(); workspace.Open(path, text, 1);
        var opened = await workspace.AnalyzeAsync(); Assert.True(opened.Analysis.Success); Assert.True(opened.ReusedProjects > 0);
        Assert.Same(disk.Analysis.Projects.Single().Result, opened.Analysis.Projects.Single().Result);
        workspace.Change(path, 2, [new(text)]); var changed = await workspace.AnalyzeAsync();
        Assert.Same(opened.Analysis.Projects.Single().Result, changed.Analysis.Projects.Single().Result);
        Assert.NotEqual(opened.Revision, changed.Revision); Assert.Equal(2, changed.Documents.Document(path).Version);
        Assert.Equal("content_modified", (await Assert.ThrowsAsync<ContextFailure>(() => workspace.VerifyAsync(opened))).Code);
        await workspace.VerifyAsync(changed);
    }

    [Fact]
    public async Task Broken_current_sources_and_changed_foreign_inputs_never_expose_old_success()
    {
        using var directory = sdk.CreateProject(); await Prepare(directory);
        var path = directory.File("Game/Bridge.fpas"); var source = File.ReadAllText(path);
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        var original = await workspace.AnalyzeAsync(); Assert.True(original.Analysis.Success);
        workspace.Open(path, "unit Broken;", 1); Assert.Null(workspace.Current);
        var broken = await workspace.AnalyzeAsync(); Assert.False(broken.Analysis.Success); Assert.Same(broken, workspace.Current);
        var partial = broken.Queries().Find(new("Broken", "exact"));
        Assert.False(partial.Coverage!.Complete); Assert.Contains(partial.Find!.Matches, declaration => declaration.Name == "Broken");
        workspace.Change(path, 2, [new(source)]); Assert.True((await workspace.AnalyzeAsync()).Analysis.Success);
        var foreign = Directory.GetFiles(directory.File("Shared"), "*.cs").First(); File.AppendAllText(foreign, "\n// changed foreign source\n");
        workspace.Invalidate(); var stale = await workspace.AnalyzeAsync();
        Assert.False(stale.Analysis.Success); Assert.Contains(stale.Analysis.Context.Issues, issue => issue.Code == "foreign_metadata_stale");
        Assert.Same(stale, workspace.Current); Assert.Throws<ContextFailure>(() => stale.Queries());
        await Prepare(directory, "Shared/Shared.csproj"); workspace.Invalidate(); Assert.True((await workspace.AnalyzeAsync()).Analysis.Success);
    }

    [Fact]
    public async Task Cancellation_supersession_and_disposal_join_owned_analysis_and_prevent_republication()
    {
        using var directory = sdk.CreateProject(); await Prepare(directory);
        var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        using var canceled = new CancellationTokenSource(); canceled.Cancel();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => workspace.AnalyzeAsync(canceled.Token));
        var work = workspace.AnalyzeAsync(); workspace.Invalidate();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => work); Assert.Null(workspace.Current);
        var fresh = await workspace.AnalyzeAsync(); Assert.True(fresh.Analysis.Success);
        workspace.Invalidate(); var pending = workspace.AnalyzeAsync(); await workspace.DisposeAsync();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => pending);
        Assert.Null(workspace.Current); await Assert.ThrowsAsync<ObjectDisposedException>(() => workspace.AnalyzeAsync()); await workspace.DisposeAsync();
        await using var limited = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")), new(RetainedBytes: 1024));
        Assert.Equal("workspace_limit", (await Assert.ThrowsAsync<ContextFailure>(() => limited.AnalyzeAsync())).Code); Assert.Null(limited.Current);
    }

    [Fact]
    public async Task Unwatched_source_changes_are_detected_before_publishing_results()
    {
        using var directory = sdk.CreateProject(); await Prepare(directory);
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        var snapshot = await workspace.AnalyzeAsync(); File.AppendAllText(directory.File("Game/Bridge.fpas"), "\n// changed\n");
        Assert.Equal("content_modified", (await Assert.ThrowsAsync<ContextFailure>(() => workspace.VerifyAsync(snapshot))).Code);
        Assert.Null(workspace.Current); Assert.NotEqual(snapshot.Revision, workspace.Revision);
        var refreshed = await workspace.AnalyzeAsync();
        var metadata = refreshed.Analysis.Context.Projects.Single().Request.References.Single(path => Path.GetFileName(path) == "Shared.dll");
        var original = File.ReadAllBytes(metadata); workspace.Open(metadata, "unit Unowned; end unit;", 1);
        var protectedInputs = await workspace.AnalyzeAsync(); File.WriteAllBytes(metadata, [.. original, 0]);
        Assert.Equal("content_modified", (await Assert.ThrowsAsync<ContextFailure>(() => workspace.VerifyAsync(protectedInputs))).Code);
        File.WriteAllBytes(metadata, original); workspace.Close(metadata);
        var fresh = await workspace.AnalyzeAsync(); File.Delete(directory.File("Game/Game.fpasproj"));
        await Assert.ThrowsAsync<ContextFailure>(() => workspace.VerifyAsync(fresh)); Assert.Null(workspace.Current);
    }

    [Fact]
    public async Task Restored_FPAS_sources_supply_dependency_metadata_without_any_built_DLLs()
    {
        using var directory = sdk.CreateProject();
        var library = directory.File("Game/Game.fpasproj");
        File.WriteAllText(library, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(directory.File("Game/Bridge.fpas"), "unit Demo.Bridge; public function Read(): int; begin return 42; end function; end unit;");
        Directory.CreateDirectory(directory.File("Consumer"));
        const string source = "unit Consumer; uses Demo.Bridge as B; public function Run(): int; begin return B.Read(); end function; end unit;";
        var path = directory.File("Consumer/Main.fpas"); File.WriteAllText(path, source);
        var project = directory.File("Consumer/Consumer.fpasproj");
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup><ItemGroup><ProjectReference Include=\"../Game/Game.fpasproj\" /></ItemGroup></Project>");
        var restored = await sdk.Run(directory, "restore", project); Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        await using var workspace = new LiveWorkspace(new(project));
        var result = await workspace.AnalyzeAsync(); Assert.True(result.Analysis.Success, string.Join(';', result.Analysis.Context.Issues.Select(issue => issue.Message)));
        Assert.Equal(library, result.Queries().Execute("definition", Select(path, source, "Read()")).Definition!.Project);
        await workspace.VerifyAsync(result);
        Assert.DoesNotContain(Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories), path => path.EndsWith(".dll", StringComparison.Ordinal) || path.EndsWith(".pdb", StringComparison.Ordinal));
    }
}
