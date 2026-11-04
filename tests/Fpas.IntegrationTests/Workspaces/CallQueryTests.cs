using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Xml.Linq;
using static Fpas.IntegrationTests.Workspaces.Support.SourcePosition;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class CallQueryTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Library = """
        unit Numerics.Api;
        uses System as S;
        public type Box<T> = record Value: T; end record;
        public type Choice = union Empty; Value(Item: int); end union;
        public type IValue = interface Value: int; function Read(): int; end interface;
        public type Sample = record implements IValue
          Value: int;
          function Read(): int; begin return self.Value; end function;
        end record;
        public function Read(Value: int): int; begin return Value; end function;
        public function Read(Value: string): int; begin return Value.Length; end function;
        public function Identity<T>(Value: T): T; begin return Value; end function;
        public function Recursive(Value: int): int;
        begin if Value = 0 then return 0; end if; return Recursive(Value - 1); end function;
        public function Apply(Value: function(int): int): int; begin return Value(1); end function;
        public function Dispatch(Value: IValue): int; begin return Value.Read() + Value.Value; end function;
        public function Make(): int;
        begin
          const Boxed: Box<int> := new Box<int>(Read(1));
          const Item: Choice := Choice.Value(Boxed.Value);
          const Empty: Choice := Choice.Empty;
          const Payload: Choice.ValueCase := cast<Choice.ValueCase>(Item);
          discard Payload.Item;
          discard Payload.GetHashCode();
          return Boxed.Value;
        end function;
        public function Empty(): int; begin return 0; end function;
        end unit;
        """;
    private const string Consumer = """
        program Consumer;
        uses Numerics.Api as A;
        begin
          discard A.Read(1);
          discard A.Read(2);
          discard A.Identity<int>(1);
          discard A.Apply(A.Read);
          discard A.Dispatch(new A.Sample(1));
          const Boxed: A.Box<int> := new A.Box<int>(A.Read(1));
          const Item: A.Choice := A.Choice.Value(Boxed.Value);
          const Empty: A.Choice := A.Choice.Empty;
          const Payload: A.Choice.ValueCase := cast<A.Choice.ValueCase>(Item);
          discard Payload.Item;
          discard Payload.GetHashCode();
        end;
        end program;
        """;

    [Fact]
    public async Task Callees_select_exact_source_bodies_and_distinguish_metadata_bodyless_and_empty()
    {
        using var project = Graph();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("First/First.fpasproj")), deadline.Token);
        var query = new SourceQuery(WorkspaceAnalysis.Analyze(context, deadline.Token));
        var selected = query.Calls("callees", Select(project, "First/Program.fpas", Consumer, "Read(1)"), cancellationToken: deadline.Token);
        Assert.Equal("available", selected.Callees!.Availability); Assert.Empty(selected.Callees.Page!.Sites);
        Assert.Equal(project.File("Game/Game.fpasproj"), selected.Callees.Target!.Project);
        var empty = query.Calls("callees", Select(project, "Game/Bridge.fpas", Library, "Empty():"));
        Assert.Equal(0, empty.Callees!.Page!.TotalCount); Assert.NotNull(empty.Callees.Summary);
        var bodyless = query.Calls("callees", Select(project, "Game/Bridge.fpas", Library, "Read(): int; end interface"));
        Assert.Equal("no_body", bodyless.Callees!.Availability); Assert.Null(bodyless.Callees.Page); Assert.Null(bodyless.Callees.Summary);
        var metadata = query.Calls("callees", Select(project, "Game/Bridge.fpas", Library, "Length;"));
        Assert.Equal("metadata_only", metadata.Callees!.Availability); Assert.Null(metadata.Callees.Page);
        var make = query.Calls("callees", Select(project, "Game/Bridge.fpas", Library, "Make():"));
        var sites = make.Callees!.Page!.Sites;
        Assert.Equal(4, sites.Count); Assert.Equal(3, sites.Count(site => site.Site.Kind == "construct"));
        var entry = query.Calls("callees", Select(project, "First/Program.fpas", Consumer, "begin"), body: true, limit: 1, offset: 1);
        Assert.Equal("program_entry", entry.Callees!.Body!.Kind); Assert.Equal(1, entry.Callees.Page!.ReturnedCount); Assert.Equal(2, entry.Callees.Page.NextOffset); Assert.True(entry.Truncated);
        var past = query.Calls("callees", Select(project, "First/Program.fpas", Consumer, "begin"), body: true, offset: int.MaxValue);
        Assert.Empty(past.Callees!.Page!.Sites); Assert.Null(past.Callees.Page.NextOffset); Assert.True(past.Truncated);
        Assert.Equal("no_symbol", query.Calls("callees", Select(project, "First/Program.fpas", Consumer, "\n")).Reason);
        Assert.Equal("no_body", query.Calls("callees", Select(project, "First/Program.fpas", Consumer, "program"), body: true).Reason);
        Assert.Equal("not_callable_target", query.Calls("callees", Select(project, "First/Program.fpas", Consumer, "Boxed:" )).Reason);
        await context.VerifyInputsAsync(deadline.Token);
    }

    [Fact]
    public async Task Callers_share_exact_sites_across_DLLs_and_keep_references_dispatch_and_scope_distinct()
    {
        using var project = Graph();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx")), deadline.Token);
        var query = new SourceQuery(WorkspaceAnalysis.Analyze(context, deadline.Token));
        var target = Select(project, "Game/Bridge.fpas", Library, "Read(Value: int");
        var incoming = query.Calls("callers", target, cancellationToken: deadline.Token);
        Assert.Equal(9, incoming.Callers!.Page.TotalCount);
        Assert.Equal(7, incoming.Callers.Page.InvocationCount); Assert.Equal(2, incoming.Callers.Page.CallableReferenceCount);
        Assert.Equal(3, incoming.Searched!.Count);
        var page = query.Calls("callers", target, limit: 2, offset: 1);
        Assert.Equal(incoming.Callers.Page.Sites.Skip(1).Take(2).Select(site => site.Site.Id), page.Callers!.Page.Sites.Select(site => site.Site.Id));
        Assert.True(page.Truncated); Assert.Equal(3, page.Callers.Page.NextOffset);
        var other = query.Calls("callers", Select(project, "Game/Bridge.fpas", Library, "Read(Value: string"));
        Assert.Equal(0, other.Callers!.Page.TotalCount);
        var generic = query.Calls("callers", Select(project, "Game/Bridge.fpas", Library, "Identity<T>"));
        Assert.Equal(2, generic.Callers!.Page.TotalCount);
        Assert.All(generic.Callers.Page.Sites, site => Assert.Equal("int", site.Types.Single(type => type.Id == site.Site.Signature!.TypeParameters[0]).Display));
        var recursive = query.Calls("callers", Select(project, "Game/Bridge.fpas", Library, "Recursive(Value:"));
        Assert.Equal("Recursive", Assert.Single(recursive.Callers!.Page.Sites).Target!.Symbol.Name);
        var contract = query.Calls("callers", Select(project, "Game/Bridge.fpas", Library, "Read(): int; end interface"));
        Assert.Single(contract.Callers!.Page.Sites); Assert.Equal("interface", contract.Callers.Page.Sites[0].Site.Dispatch);
        var implementation = query.Calls("callers", Select(project, "Game/Bridge.fpas", Library, "Read(): int; begin"));
        Assert.Empty(implementation.Callers!.Page.Sites);
        var dispatch = query.Calls("callees", Select(project, "Game/Bridge.fpas", Library, "Dispatch(Value:"));
        Assert.Equal(["invoke", "property_get"], dispatch.Callees!.Page!.Sites.Select(site => site.Site.Kind).Order(StringComparer.Ordinal));
        Assert.True(incoming.Callers.Summary.Unresolved["runtime_delegate_target"] > 0);
        var source = query.Calls("callees", Select(project, "Game/Bridge.fpas", Library, "Make():")).Callees!.Page!.Sites;
        var binary = query.Calls("callees", Select(project, "First/Program.fpas", Consumer, "begin"), body: true).Callees!.Page!.Sites;
        Assert.Equal(4, binary.Count(site => site.Site.Kind == "construct" && site.Target?.Origin == "source"));
        Assert.DoesNotContain(source.Concat(binary).Select(site => site.Site.Operation?.Name), name => name is "get_Value" or "get_Item" or "GetHashCode" or "<With:Value>");
        Assert.Equal(1, query.Calls("callees", Select(project, "Game/Bridge.fpas", Library, "Make():")).Callees!.Summary!.Excluded["generated_infrastructure"]);
        await context.VerifyInputsAsync(deadline.Token);
    }

    [Fact]
    public async Task Anonymous_bodies_use_original_UTF16_regions_and_initializer_bodies_have_no_incoming_callable()
    {
        using var project = sdk.CreateProject();
        const string text = """
            unit Regions;
            public const Initial: int := Seed();
            public const Factory: function(): int := function(): int => Seed();
            public function Seed(): int; begin return 1; end function;
            public function Work(): int;
            begin
              const Prefix: string := '😀';
              const Callback: function(): int := function(): int begin return Seed(); end function;
              return Callback();
            end function;
            end unit;
            """;
        var source = text.Replace("\n", "\r\n", StringComparison.Ordinal);
        File.WriteAllText(project.File("Game/Bridge.fpas"), source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")), deadline.Token);
        var query = new SourceQuery(WorkspaceAnalysis.Analyze(context, deadline.Token));
        var position = Select(project, "Game/Bridge.fpas", source, "return Seed()");
        var outgoing = query.Calls("callees", position, body: true);
        Assert.Equal("lambda", outgoing.Callees!.Body!.Kind); Assert.Null(outgoing.Callees.Target);
        Assert.Equal("Seed", Assert.Single(outgoing.Callees.Page!.Sites).Target!.Symbol.Name);
        var incoming = query.Calls("callers", position, body: true);
        Assert.Equal("callable_reference", Assert.Single(incoming.Callers!.Page.Sites).Relationship);
        Assert.Equal("lambda", incoming.Callers.Body!.Kind); Assert.Null(incoming.Callers.Target);
        var factory = Select(project, "Game/Bridge.fpas", source, "=> Seed()");
        Assert.Equal("lambda", query.Calls("callees", factory, body: true).Callees!.Body!.Kind);
        Assert.Equal("callable_reference", Assert.Single(query.Calls("callers", factory, body: true).Callers!.Page.Sites).Relationship);
        var initializer = Select(project, "Game/Bridge.fpas", source, "Seed();");
        Assert.Equal("global_initializer", query.Calls("callees", initializer, body: true).Callees!.Body!.Kind);
        Assert.Equal("not_callable_target", query.Calls("callers", initializer, body: true).Reason);
        var end = Select(project, "Game/Bridge.fpas", source, ";\r\n  return Callback");
        Assert.Equal("routine", query.Calls("callees", end, body: true).Callees!.Body!.Kind);
        await context.VerifyInputsAsync(deadline.Token);
    }

    [Fact]
    public async Task Constructor_and_accessor_anchors_select_exact_overloads_and_operations()
    {
        using var project = sdk.CreateProject();
        const string source = """
            unit Targets;
            uses System.Text as T;
            public procedure Work();
            begin
              const First: T.StringBuilder := new T.StringBuilder();
              const Second: T.StringBuilder := new T.StringBuilder(8);
              First.Length := Second.Length;
            end procedure;
            end unit;
            """;
        File.WriteAllText(project.File("Game/Bridge.fpas"), source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(3));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")), deadline.Token);
        var query = new SourceQuery(WorkspaceAnalysis.Analyze(context, deadline.Token));
        foreach (var (marker, method, arity) in new[] { ("StringBuilder();", ".ctor", 0), ("StringBuilder(8)", ".ctor", 1), ("Length :=", "set_Length", 1), ("Length;", "get_Length", 0) })
        {
            var result = query.Calls("callers", Select(project, "Game/Bridge.fpas", source, marker));
            var site = Assert.Single(result.Callers!.Page.Sites);
            Assert.Equal(method, result.Callers.SelectedOperation!.Name); Assert.Equal(method, site.Site.Operation!.Name);
            Assert.Equal(arity, site.Site.Operation.Parameters.Count);
        }
        await context.VerifyInputsAsync(deadline.Token);
    }

    private TemporaryDirectory Graph()
    {
        var project = sdk.CreateProject(); File.WriteAllText(project.File("Game/Bridge.fpas"), Library);
        var solution = XDocument.Load(project.File("Game.slnx"));
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
        solution.Save(project.File("Game.slnx")); return project;
    }
}
