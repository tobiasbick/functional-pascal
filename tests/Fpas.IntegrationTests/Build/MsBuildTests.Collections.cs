using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Directory_inventory_builds_and_counts_files_with_real_arguments()
    {
        using var project = sdk.CreateProject();
        var fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "directory-inventory");
        Directory.CreateDirectory(project.File("Inventory"));
        foreach (var file in Directory.GetFiles(fixture))
            File.WriteAllText(project.File("Inventory/" + Path.GetFileName(file)), File.ReadAllText(file).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Inventory/DirectoryInventory.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var input = project.File("text files Grüße");
        Directory.CreateDirectory(Path.Combine(input, "nested"));
        File.WriteAllText(Path.Combine(input, "a.txt"), "first\nsecond\nthird");
        File.WriteAllText(Path.Combine(input, "empty.txt"), "");
        File.WriteAllText(Path.Combine(input, "nested", "ü.txt"), "text\n");
        File.WriteAllText(Path.Combine(input, "skip.bin"), "ignored\n");
        File.WriteAllText(Path.Combine(input, ".hidden.txt"), "excluded\n");
        const string executable = "Inventory/bin/Debug/net10.0/DirectoryInventory.dll";
        var run = await sdk.Run(project, executable, input, "*.txt");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "a.txt: 3", "empty.txt: 0", Path.Combine("nested", "ü.txt") + ": 1", "Files: 3", "Lines: 4", ""), run.Output);
        Assert.Equal(2, (await sdk.Run(project, executable)).ExitCode);
        Assert.Equal(1, (await sdk.Run(project, executable, project.File("missing"))).ExitCode);
    }

    [Fact]
    public async Task Imported_constraints_and_nullable_element_contracts_are_checked()
    {
        using var project = sdk.CreateProject();
        var build = await sdk.Run(project, "build", "Shared/Shared.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var cases = new (string Body, string? Error)[]
        {
            ("const X: D.NeedsClass<int> := new D.NeedsClass<int>();", "FP3018"),
            ("const X: D.NeedsValue<string> := new D.NeedsValue<string>();", "FP3018"),
            ("const X: D.NeedsValue<int?> := new D.NeedsValue<int?>();", "FP3018"),
            ("const X: D.NeedsNew<string> := new D.NeedsNew<string>();", "FP3018"),
            ("const X: D.NeedsDisposable<string> := new D.NeedsDisposable<string>();", "FP3018"),
            ("const X: D.NeedsComparable<object> := new D.NeedsComparable<object>();", "FP3018"),
            ("const X: D.NeedsComparable<int> := new D.NeedsComparable<int>();", null),
            ("const X: D.NeedsNew<int> := new D.NeedsNew<int>();", null),
            ("const X: D.NeedsDisposable<D.Resource> := new D.NeedsDisposable<D.Resource>();", null),
            ("const X: string := D.CollectionApi.NullableArray()[0];", "FP3013"),
            ("const X: string := D.CollectionApi.Nested()[0][0];", "FP3013"),
            ("const X: string := D.CollectionApi.NullableList(true)[0];", "FP3013"),
            ("const X: string := D.CollectionApi.NonNullArray()[0];", null),
            ("const X: D.Box<int> := new D.Box<int>(1); const N: int := X.Maybe();", null)
        };
        foreach (var (body, error) in cases)
        {
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Check.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"],
                "program Check; uses Demo as D; begin " + body + " end; end program;");
            if (error is null) Assert.True(result.ExitCode == 0, body + ": " + result.Output + result.Error);
            else
            {
                Assert.NotEqual(0, result.ExitCode);
                Assert.Contains("\"code\":\"" + error + "\"", result.Output + result.Error);
            }
        }
    }

    [Fact]
    public async Task Enumeration_disposes_on_all_exits_and_failed_operations()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses Demo as D, System as Sys, System.Collections as Legacy;
            public function Run(Name: string; Mode: int): int;
            begin
              using const Outer: D.Resource := new D.Resource('outer');
              const Source: D.Probe<int> := new D.Probe<int>(Name, [1, 2]);
              for Item: int in Source do
                if Mode = 1 then break; end if;
                if Mode = 2 then continue; end if;
                if Mode = 3 then return Item; end if;
                if Mode = 4 then raise new Sys.InvalidOperationException('body'); end if;
              end for;
              return 0;
            end function;
            public procedure Old(Disposable: bool);
            begin
              const Source: Legacy.IEnumerable := new D.Legacy(Disposable);
              for Item: object? in Source do D.Resource.Note(cast<string>(Item ?? 'null')); end for;
            end procedure;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            static void Check(string name, int mode, string expected, bool throws = false)
            {
                Resource.Reset();
                try { var value = Bridge.Run(name, mode); if (throws || value != (mode == 3 ? 1 : 0)) throw new Exception("Wrong result"); }
                catch (InvalidOperationException) when (throws) { }
                if (Resource.Log != expected) throw new Exception(name + "/" + mode + ": " + Resource.Log);
            }
            const string open = "open:outer;acquire:ok;";
            const string one = "move:ok;current:ok;";
            const string close = "dispose:ok;close:outer;";
            Check("ok", 0, open + one + one + "move:ok;" + close);
            Check("ok", 1, open + one + close);
            Check("ok", 2, open + one + one + "move:ok;" + close);
            Check("ok", 3, open + one + close);
            Check("ok", 4, open + one + close, true);
            Check("acquire-error", 0, "open:outer;acquire:acquire-error;close:outer;", true);
            Check("move-error", 0, "open:outer;acquire:move-error;move:move-error;dispose:move-error;close:outer;", true);
            Check("current-error", 0, "open:outer;acquire:current-error;move:current-error;current:current-error;dispose:current-error;close:outer;", true);
            Check("dispose-error", 1, "open:outer;acquire:dispose-error;move:dispose-error;current:dispose-error;dispose:dispose-error;close:outer;", true);
            Resource.Reset(); Bridge.Old(false);
            if (Resource.Log != "null;legacy;") throw new Exception(Resource.Log);
            Resource.Reset(); Bridge.Old(true);
            if (Resource.Log != "null;legacy;legacy-dispose;") throw new Exception(Resource.Log);
            Console.WriteLine("Enumeration cleanup verified");
            """);
        await BuildAndRunCollections(project, "Enumeration cleanup verified");
    }

    [Fact]
    public async Task Nested_nullable_arrays_and_generic_signatures_round_trip_through_CSharp_and_Fpas()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses System.Collections.Generic as C, Demo as D;
            public function Echo(Value: C.Dictionary<string, C.List<array of string?>>): C.Dictionary<string, C.List<array of string?>>;
            begin return D.CollectionApi.Echo(Value); end function;
            public function Maybe(Value: C.List<string?>?): C.List<string?>?;
            begin return Value; end function;
            public function Grid(Value: array[,] of int): array[,] of int;
            begin Value[1, 0] := 9; return Value; end function;
            public function Jagged(Value: array of array of string?): array of array of string?;
            begin return Value; end function;
            public function Pair(Value: C.KeyValuePair<string, string?>?): C.KeyValuePair<string, string?>?;
            begin return D.CollectionApi.EchoPair(Value); end function;
            public function Read(): string;
            begin
              const Box: D.DerivedBox<string?> := new D.DerivedBox<string?>(null);
              Box.Field := 'field'; Box.Value := null;
              const Cells: array of string? := Box.Repeat();
              const Optional: C.List<string?>? := D.CollectionApi.NullableList(true);
              if Optional = null then return 'missing'; end if;
              const Elements: C.List<array of string?> := D.CollectionApi.Nested();
              const Numbers: C.List<int?> := D.CollectionApi.NullableNumbers();
              return (Cells[1] ?? 'null') + ':' + (Optional[0] ?? 'null') + ':' + (Elements[0][0] ?? 'null') + ':' +
                D.CollectionApi.Overload(['item']) + ':' + (Box.Maybe() ?? 'null');
            end function;
            public function StructArrays(): int;
            begin
              const Vector: array of D.Counter := new D.Counter[1];
              Vector[0].Increment(); Vector[0].Value := Vector[0].Value + 2;
              const Grid: array[,] of D.Counter := new D.Counter[1, 1];
              Grid[0, 0].Increment(); Grid[0, 0].Value := Grid[0, 0].Value + 3;
              return Vector[0].Value + Grid[0, 0].Value;
            end function;
            public procedure IndexOrder();
            begin D.IndexProbe.Receiver()[D.IndexProbe.Index(1), D.IndexProbe.Index(2)] := D.IndexProbe.Value(); end procedure;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using System.Reflection;
            using Demo;
            var values = new Dictionary<string, List<string?[]>> { ["key"] = [new string?[] { null, "text" }] };
            if (!ReferenceEquals(values, Bridge.Echo(values))) throw new Exception("Array/list identity");
            if (Bridge.Maybe(null) is not null) throw new Exception("Nullable root");
            var grid = new int[2, 1];
            if (!ReferenceEquals(grid, Bridge.Grid(grid)) || grid[1, 0] != 9) throw new Exception("Rectangle");
            string?[][] jagged = [[null]];
            if (!ReferenceEquals(jagged, Bridge.Jagged(jagged))) throw new Exception("Jagged");
            var pair = Bridge.Pair(new KeyValuePair<string, string?>("key", null));
            if (!pair.HasValue || pair.Value.Key != "key" || pair.Value.Value is not null) throw new Exception("Nullable generic struct");
            if (Bridge.Read() != "null:null:null:sequence:null") throw new Exception(Bridge.Read());
            if (Bridge.StructArrays() != 7) throw new Exception("Writable struct array storage");
            Bridge.IndexOrder();
            if (IndexProbe.Log != "receiver;index:1;index:2;value;set:1:2:9;") throw new Exception(IndexProbe.Log);
            var info = new NullabilityInfoContext().Create(typeof(Bridge).GetMethod("Echo")!.ReturnParameter);
            if (info.ReadState != NullabilityState.NotNull ||
                info.GenericTypeArguments[1].GenericTypeArguments[0].ElementType!.ReadState != NullabilityState.Nullable)
                throw new Exception("Nested nullable metadata");
            var parameter = new NullabilityInfoContext().Create(typeof(Bridge).GetMethod("Maybe")!.GetParameters()[0]);
            if (parameter.ReadState != NullabilityState.Nullable || parameter.GenericTypeArguments[0].ReadState != NullabilityState.Nullable)
                throw new Exception("Nullable generic metadata");
            static string Shape(NullabilityInfo info) => info.Type.Name + ":" + info.ReadState + "[" + string.Join(",", info.GenericTypeArguments.Select(Shape)) + "]";
            var context = new NullabilityInfoContext();
            if (Shape(context.Create(typeof(Bridge).GetMethod("Pair")!.ReturnParameter)) !=
                Shape(context.Create(typeof(CollectionApi).GetMethod("EchoPair")!.ReturnParameter)))
                throw new Exception("Nullable generic struct annotations");
            Console.WriteLine("Generic metadata verified");
            """);
        await BuildAndRunCollections(project, "Generic metadata verified");
        Directory.CreateDirectory(project.File("Middle"));
        File.WriteAllText(project.File("Middle/Middle.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Middle/Api.fpas"), """
            unit Demo.Middle;
            uses Demo.Bridge as B, System.Collections.Generic as C;
            public function Read(): string;
            begin
              const Data: C.Dictionary<string, C.List<array of string?>> := new C.Dictionary<string, C.List<array of string?>>();
              const Rows: array of array of string? := [[null]];
              Data['key'] := new C.List<array of string?>(Rows);
              const Returned: C.Dictionary<string, C.List<array of string?>> := B.Echo(Data);
              return Returned['key'][0][0] ?? 'round trip';
            end function;
            end unit;
            """);
        var host = project.File("Host/Host.csproj");
        File.WriteAllText(host, File.ReadAllText(host).Replace("../Game/Game.fpasproj", "../Middle/Middle.fpasproj", StringComparison.Ordinal));
        File.WriteAllText(project.File("Host/Program.cs"), "Console.WriteLine(Demo.Middle.Read());");
        await BuildAndRunCollections(project, "round trip");
    }

    private async Task BuildAndRunCollections(TemporaryDirectory project, string expected)
    {
        var build = await sdk.Run(project, "build", "Host/Host.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(expected + Environment.NewLine, run.Output);
    }
}
