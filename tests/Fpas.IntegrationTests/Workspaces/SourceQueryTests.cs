using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Xml.Linq;
using static Fpas.IntegrationTests.Workspaces.Support.SourcePosition;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class SourceQueryTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Library = """
        unit Demo.Bridge;
        public type Box<T> = record Value: T; end record;
        public type Number = int;
        public function Read(Value: int): int; begin return Value; end function;
        public function Unused(): int; begin return 0; end function;
        public function Identity<T>(Input: T): T; begin return Input; end function;
        end unit;
        """;
    private const string Consumer = """
        program Consumer;
        uses Demo.Bridge as B;
        begin
          const Box: B.Box<int> := new B.Box<int>(Value := B.Read(Value := 42));
          const Value: B.Number := Box.Value;
          const Roundtrip: int := B.Identity<int>(Value);
        end;
        end program;
        """;

    [Fact]
    public async Task Find_searches_checked_declarations_with_literal_names_project_filters_and_stable_pages()
    {
        using var project = CreateGraph();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx")), deadline.Token);
        var query = new SourceQuery(WorkspaceAnalysis.Analyze(context, deadline.Token));
        var exact = query.Find(new("Read", "exact"), deadline.Token);
        Assert.Equal("Read", Assert.Single(exact.Find!.Matches).Name);
        Assert.Equal("Demo.Bridge.Read", exact.Find.Matches[0].QualifiedName);
        Assert.Equal(project.File("Game/Game.fpasproj"), exact.Find.Matches[0].Project);
        Assert.Null(exact.Selection); Assert.False(exact.Truncated);
        Assert.Empty(query.Find(new("read", "exact")).Find!.Matches);
        Assert.Empty(query.Find(new("Read", "exact", ["local"])).Find!.Matches);
        Assert.Single(query.Find(new("Demo.Bridge.Re", "prefix", ["routine"])).Find!.Matches);
        var all = query.Find(new("Value", "contains"));
        Assert.True(all.Find!.TotalCount > 2);
        var page = query.Find(new("Value", Limit: 1, Offset: 1));
        Assert.Equal(all.Find.Matches[1], Assert.Single(page.Find!.Matches));
        Assert.Equal(all.Find.TotalCount, page.Find.TotalCount); Assert.Equal(2, page.Find.NextOffset); Assert.True(page.Truncated);
        var past = query.Find(new("Value", Offset: int.MaxValue));
        Assert.Empty(past.Find!.Matches); Assert.Null(past.Find.NextOffset); Assert.True(past.Truncated);
        var filtered = query.Find(new("Value", Project: project.File("First/First.fpasproj")));
        Assert.Equal([project.File("First/First.fpasproj")], filtered.Searched);
        Assert.All(filtered.Find!.Matches, match => Assert.Equal(project.File("First/First.fpasproj"), match.Project));
        Assert.Equal("invalid_argument", Assert.Throws<ContextFailure>(() => query.Find(new("Value", Project: project.File("Missing.fpasproj")))).Code);
        Assert.Equal("invalid_argument", Assert.Throws<ContextFailure>(() => query.Find(new("Value", Kinds: ["missing"])) ).Code);
        await context.VerifyInputsAsync(deadline.Token);
    }

    [Fact]
    public async Task Inspect_returns_exact_source_direct_types_metadata_and_explicit_omissions()
    {
        using var project = CreateGraph();
        const string declaration = "public function Wrap(Value: Box<int>; Count: Number; Link: Node): Box<int>; begin const Temporary: string := 'body only'; return Value; end function;";
        var library = Library.Replace("end unit;", "public type Node = record Next: Node?; end record;\r\n// Wrap documentation\r\n" + declaration + "\r\nend unit;", StringComparison.Ordinal);
        File.WriteAllText(project.File("Game/Bridge.fpas"), library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx")), deadline.Token);
        var query = new SourceQuery(WorkspaceAnalysis.Analyze(context, deadline.Token));
        var selection = Select(project, "Game/Bridge.fpas", library, "Wrap(Value:");
        var result = query.Inspect(selection, cancellationToken: deadline.Token);
        Assert.Equal("ok", result.Status); Assert.False(result.Truncated);
        var inspection = result.Inspect!;
        Assert.Equal(declaration, inspection.Source.Declaration.Text);
        Assert.Equal("// Wrap documentation", inspection.Source.Documentation.Text);
        Assert.Equal("location", inspection.Source.Body.Availability); Assert.Null(inspection.Source.Body.Text);
        Assert.Equal("Demo.Bridge", Assert.Single(inspection.ContainingDeclarations).Name);
        Assert.Contains(inspection.Related, item => item.Definition.Symbol.Name == "Box");
        Assert.Contains(inspection.Related, item => item.Definition.Symbol.Name == "Number");
        Assert.Contains(inspection.Related, item => item.Definition.Symbol.Name == "Node");
        var recursive = query.Inspect(Select(project, "Game/Bridge.fpas", library, "Node = record"));
        Assert.DoesNotContain(recursive.Inspect!.Related, item => item.Definition.Symbol.Name == "Node");
        Assert.DoesNotContain(inspection.Types, type => type.Display == "string");
        Assert.Contains(inspection.Types, type => type.Display == "int");
        var limited = query.Inspect(selection, 1);
        Assert.True(limited.Truncated); Assert.Equal(1, limited.Inspect!.ReturnedRelatedSourceCount);
        Assert.True(limited.Inspect.Omissions.RelatedLimit > 0);
        var field = query.Inspect(Select(project, "First/Program.fpas", Consumer, "Value;"));
        Assert.Equal("Value: T;", field.Inspect!.Source.Declaration.Text);
        Assert.Equal("no_body", field.Inspect.Source.Body.Availability);
        var metadata = query.Inspect(Select(project, "First/Program.fpas", Consumer, "int>"));
        Assert.Equal("metadata", metadata.Inspect!.Definition.Origin);
        Assert.Equal("metadata_only", metadata.Inspect.Source.Declaration.Availability);
        Assert.Null(metadata.Inspect.Source.Declaration.Span);
        await context.VerifyInputsAsync(deadline.Token);
    }

    [Fact]
    public async Task Inspect_omits_oversized_whole_source_slices_without_hiding_analysis_coverage()
    {
        using var project = sdk.CreateProject();
        var source = "unit Large; public function Read(): int; begin // " + string.Concat(Enumerable.Repeat("🙂", 20_000)) + "\n return 42; end function; end unit;";
        File.WriteAllText(project.File("Game/Bridge.fpas"), source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")), deadline.Token);
        var analysis = WorkspaceAnalysis.Analyze(context, deadline.Token);
        var result = new SourceQuery(analysis).Inspect(Select(project, "Game/Bridge.fpas", source, "Read()"));
        Assert.True(analysis.IsComplete); Assert.True(result.Truncated);
        Assert.Equal("source_budget", result.Inspect!.Source.Declaration.Availability);
        Assert.Null(result.Inspect.Source.Declaration.Text); Assert.NotNull(result.Inspect.Source.Declaration.Span);
        Assert.Equal(1, result.Inspect.Omissions.SourceBudget);
        Assert.Equal("Read", result.Inspect.Definition.Symbol.Name);
    }

    [Fact]
    public async Task Definitions_bridge_exact_generic_properties_aliases_and_named_parameters_to_prepared_FPAs_source()
    {
        using var project = CreateGraph();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx")), deadline.Token);
        Assert.Empty(context.Issues);
        Assert.Contains(context.DependencyOutputs, output => output.Project == project.File("Game/Game.fpasproj") && Path.GetFileName(output.Path) == "Game.dll");
        var analysis = WorkspaceAnalysis.Analyze(context, deadline.Token); Assert.True(analysis.Success, string.Join(';', analysis.Projects));
        var query = new SourceQuery(analysis);
        foreach (var (marker, kind, name) in new[] { ("Read(Value :=", "routine", "Read"), ("Number :=", "alias", "Number"),
            ("Value;", "property", "Value"), ("Value := 42", "parameter", "Value"), ("Identity<int>", "routine", "Identity") })
        {
            var result = query.Execute("definition", Select(project, "First/Program.fpas", Consumer, marker), cancellationToken: deadline.Token);
            Assert.Equal("ok", result.Status); Assert.Equal(project.File("Game/Game.fpasproj"), result.Definition!.Project);
            Assert.Equal("source", result.Definition.Origin); Assert.Equal(kind, result.Definition.Symbol.Kind); Assert.Equal(name, result.Definition.Symbol.Name);
            Assert.Equal(project.File("Game/Bridge.fpas"), result.Definition.Symbol.Declaration!.Path);
        }
        var inspected = query.Execute("type", Select(project, "First/Program.fpas", Consumer, "Box.Value"), cancellationToken: deadline.Token);
        Assert.Equal("record", inspected.Type!.Type.Kind);
        Assert.Single(inspected.Type.Type.Arguments);
        Assert.Contains(inspected.Type.Types, type => type.Id == inspected.Type.Type.Arguments[0] && type.Display == "int");
        await context.VerifyInputsAsync(deadline.Token);
    }

    [Fact]
    public async Task Refs_cover_solution_consumers_exactly_and_report_limits_and_complete_zero_results()
    {
        using var project = CreateGraph();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx")), deadline.Token);
        var analysis = WorkspaceAnalysis.Analyze(context, deadline.Token); Assert.True(analysis.Success, string.Join(';', analysis.Projects));
        var query = new SourceQuery(analysis);
        var selection = Select(project, "Game/Bridge.fpas", Library, "Read(Value:");
        var result = query.Execute("refs", selection, cancellationToken: deadline.Token);
        Assert.Equal(2, result.Refs!.ReturnedCount); Assert.False(result.Truncated);
        Assert.Equal(new[] { project.File("First/First.fpasproj"), project.File("Second/Second.fpasproj") }, result.Refs.References.Select(reference => reference.Project));
        Assert.All(result.Refs.References, reference => Assert.Equal("read", reference.Role));
        var limited = query.Execute("refs", selection, 1, deadline.Token);
        Assert.True(limited.Truncated); Assert.Single(limited.Refs!.References);
        Assert.Equal(result.Refs.References[0], limited.Refs.References[0]);
        var unused = query.Execute("refs", Select(project, "Game/Bridge.fpas", Library, "Unused()"), cancellationToken: deadline.Token);
        Assert.Equal("ok", unused.Status); Assert.Equal(0, unused.Refs!.ReturnedCount); Assert.False(unused.Truncated);
        var parameter = query.Execute("refs", Select(project, "Game/Bridge.fpas", Library, "Value: int"), cancellationToken: deadline.Token);
        Assert.Equal(3, parameter.Refs!.ReturnedCount);
        Assert.Equal(2, parameter.Refs.References.Count(reference => reference.Role == "argument"));
        Assert.NotEmpty(result.BinaryBoundaries);
    }

    [Fact]
    public async Task Linked_file_ownership_requires_explicit_project_and_does_not_merge_unrelated_compilations()
    {
        using var project = CreateGraph();
        const string linked = "unit Linked; function Read(): int; begin return 1; end function; function Use(): int; begin return Read(); end function; end unit;";
        File.WriteAllText(project.File("Linked.fpas"), linked);
        foreach (var name in new[] { "First", "Second" })
        {
            var path = project.File(name + "/" + name + ".fpasproj"); var xml = XDocument.Load(path);
            xml.Root!.Add(new XElement("ItemGroup", new XElement("Compile", new XAttribute("Include", "../Linked.fpas")))); xml.Save(path);
        }
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx")), deadline.Token);
        var analysis = WorkspaceAnalysis.Analyze(context, deadline.Token); Assert.True(analysis.Success, string.Join(';', analysis.Projects));
        var query = new SourceQuery(analysis); var selection = Select(project, "Linked.fpas", linked, "Read():");
        Assert.Equal(2, query.Find(new("Linked.Read", "exact")).Find!.TotalCount);
        Assert.Equal("ambiguous_project", Assert.Throws<ContextFailure>(() => query.Inspect(selection)).Code);
        Assert.Equal("ambiguous_project", Assert.Throws<ContextFailure>(() => query.Execute("refs", selection)).Code);
        Assert.Equal("ambiguous_project", Assert.Throws<ContextFailure>(() => query.Calls("callees", selection)).Code);
        Assert.Equal("ambiguous_project", Assert.Throws<ContextFailure>(() => query.Calls("callers", selection)).Code);
        var result = query.Execute("refs", selection with { Project = project.File("First/First.fpasproj") }, cancellationToken: deadline.Token);
        Assert.Equal(project.File("First/First.fpasproj"), Assert.Single(result.Refs!.References).Project);
        var calls = query.Calls("callers", selection with { Project = project.File("First/First.fpasproj") }, cancellationToken: deadline.Token);
        Assert.Equal(project.File("First/First.fpasproj"), Assert.Single(calls.Callers!.Page.Sites).Project);
    }

    [Fact]
    public async Task Edited_dependency_source_is_rebuilt_and_an_unrelated_binary_cannot_be_attached_by_signature()
    {
        using var project = CreateGraph();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var options = new ContextOptions(project.File("First/First.fpasproj"));
        var original = await WorkspaceLoader.LoadAsync(options, deadline.Token);
        Assert.Empty(original.Issues);
        var revised = Library.Replace("return Value;", "return Value + 1;", StringComparison.Ordinal);
        File.WriteAllText(project.File("Game/Bridge.fpas"), revised);
        Assert.Equal("inputs_changed", (await Assert.ThrowsAsync<ContextFailure>(() => original.VerifyInputsAsync(deadline.Token))).Code);
        var fresh = await WorkspaceLoader.LoadAsync(options with { NoRestore = true }, deadline.Token);
        Assert.Empty(fresh.Issues); Assert.NotEqual(original.SnapshotId, fresh.SnapshotId);
        var path = project.File("Game/bin/Debug/net10.0/Game.dll"); var expected = await File.ReadAllBytesAsync(path, deadline.Token);
        var input = fresh.Projects.Single(input => input.Path == project.File("Game/Game.fpasproj")).Request;
        var unrelated = Fpas.Compiler.FpasCompiler.Compile(new([new(project.File("Game/Bridge.fpas"), Library.Replace("return Value;", "return Value + 999;", StringComparison.Ordinal))],
            input.AssemblyName, input.References, Fpas.Compiler.CompilationTarget.Library));
        Assert.True(unrelated.Success); File.WriteAllBytes(path, unrelated.Program!.AssemblyImage);
        File.SetLastWriteTimeUtc(path, DateTime.UtcNow.AddMinutes(5));
        try
        {
            await WorkspaceLoader.LoadAsync(options with { NoRestore = true }, deadline.Token);
            Assert.Equal(expected, await File.ReadAllBytesAsync(path, deadline.Token));
        }
        catch (ContextFailure failure) { Assert.Equal("inputs_changed", failure.Code); }
    }

    [Fact]
    public async Task Binary_definitions_with_identical_names_and_versions_but_different_contents_are_distinct()
    {
        using var project = CreateGraph();
        const string source = "program Consumer; uses Binary.Api as Api; begin discard Api.Read(); end; end program;";
        var references = (await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")))).Projects.Single().Request.References;
        Directory.CreateDirectory(project.File("Binaries"));
        foreach (var (name, value) in new[] { ("First", 1), ("Second", 2) })
        {
            var compiled = Fpas.Compiler.FpasCompiler.Compile(new([new(name + ".fpas", "unit Binary.Api; public function Read(): int; begin return " + value + "; end function; end unit;")],
                "BinaryApi", references, Fpas.Compiler.CompilationTarget.Library));
            Assert.True(compiled.Success);
            File.WriteAllBytes(project.File("Binaries/" + name + ".dll"), compiled.Program!.AssemblyImage);
            File.WriteAllText(project.File(name + "/Program.fpas"), source);
            var path = project.File(name + "/" + name + ".fpasproj"); var xml = XDocument.Load(path);
            xml.Root!.Add(new XElement("ItemGroup", new XElement("Reference", new XAttribute("Include", "BinaryApi"), new XAttribute("HintPath", "../Binaries/" + name + ".dll")))); xml.Save(path);
        }
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx")), deadline.Token);
        var analysis = WorkspaceAnalysis.Analyze(context, deadline.Token); Assert.True(analysis.Success, string.Join(';', analysis.Projects));
        var result = new SourceQuery(analysis).Execute("refs", Select(project, "First/Program.fpas", source, "Read()"), cancellationToken: deadline.Token);
        Assert.Equal("metadata", result.Refs!.Definition.Origin);
        Assert.Equal(project.File("First/First.fpasproj"), Assert.Single(result.Refs.References).Project);
        var callers = new SourceQuery(analysis).Calls("callers", Select(project, "First/Program.fpas", source, "Read()"), cancellationToken: deadline.Token);
        Assert.Equal("metadata", callers.Callers!.Target!.Origin);
        Assert.Equal(project.File("First/First.fpasproj"), Assert.Single(callers.Callers.Page.Sites).Project);
    }

    private TemporaryDirectory CreateGraph()
    {
        var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), Library);
        var solutionPath = project.File("Game.slnx"); var solution = XDocument.Load(solutionPath);
        foreach (var name in new[] { "First", "Second" })
        {
            Directory.CreateDirectory(project.File(name)); File.WriteAllText(project.File(name + "/Program.fpas"), Consumer);
            File.WriteAllText(project.File(name + "/" + name + ".fpasproj"), $"""
                <Project Sdk="Fpas.NET.Sdk/{sdk.Version}"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup>
                  <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
                </Project>
                """);
            solution.Root!.Add(new XElement("Project", new XAttribute("Path", name + "/" + name + ".fpasproj")));
        }
        solution.Save(solutionPath); return project;
    }

}
