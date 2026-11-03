using System.Text.Json;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    private const string NestedTypeReference = """
        namespace Shared.Nesting;
        public class Outer<T> where T : class?
        {
            public class Inner<U>(T first, U second) where U : struct
            {
                public T First { get; } = first;
                public U Second { get; } = second;
                public V Echo<V>(V value) => value;
                public static Inner<U> Create(T first, U second) => new(first, second);
                public sealed class Deep<V>(V value) where V : class?
                {
                    public V Value { get; } = value;
                }
            }
            public sealed class Leaf(T value) { public T Value { get; } = value; }
            public struct Cell(T value) { public T Value { get; set; } = value; }
            public delegate U Mapper<U>(T value) where U : struct;
            public sealed class Related<U> where U : T { }
            private sealed class Hidden { }
        }
        public static class Plain
        {
            public sealed class Inner<T>(T value) { public T Value { get; } = value; }
        }
        public static class Oracle
        {
            /// <summary>Exchange.</summary>
            public static Outer<T>.Inner<U> Exchange<T, U>(ref Outer<T>.Inner<U> value)
                where T : class? where U : struct => value;
        }
        """;

    [Fact]
    public async Task Nested_generic_type_paths_preserve_CLR_identity_across_CSharp_and_Fpas()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Nested.cs"), NestedTypeReference);
        foreach (var path in new[] { "Shared/Shared.csproj", "Game/Game.fpasproj" })
        {
            var definition = XDocument.Load(project.File(path));
            definition.Root!.Add(new XElement("PropertyGroup", new XElement("GenerateDocumentationFile", "true"), new XElement("NoWarn", "1591")));
            definition.Save(project.File(path));
        }
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Nested;
            uses Shared as S, Shared.Nesting as N, System as Sys, System.Collections.Generic as C;
            public type Selected = N.Outer<string?>.Inner<int>;
            type Root = N.Outer<string?>;
            public function Create<T, U>(First: T; Second: U): S.Nesting.Outer<T>.Inner<U>;
            where T: class?; where U: struct;
            begin return new S.Nesting.Outer<T>.Inner<U>(First, Second); end function;
            // Exchange.
            public function Exchange<T, U>(ref Value: N.Outer<T>.Inner<U>): N.Outer<T>.Inner<U>;
            where T: class?; where U: struct;
            begin return Value; end function;
            public function Select(): Selected;
            begin
              const Factory: function(string?, int): Selected := N.Outer<string?>.Inner<int>.Create;
              const Value: Root.Inner<int> := Factory(null, 42);
              return Value;
            end function;
            public function Deep<T, U, V>(Value: V): N.Outer<T>.Inner<U>.Deep<V>;
            where T: class?; where U: struct; where V: class?;
            begin return new N.Outer<T>.Inner<U>.Deep<V>(Value); end function;
            public function Leaf(Value: string?): N.Outer<string?>.Leaf;
            begin return new N.Outer<string?>.Leaf(Value); end function;
            public function Cell(Value: string?): N.Outer<string?>.Cell;
            begin
              var Item: N.Outer<string?>.Cell := new N.Outer<string?>.Cell('initial');
              Item.Value := Value;
              return Item;
            end function;
            public function Map(): int;
            begin
              const Handler: N.Outer<string>.Mapper<int> := function(Text: string): int => Text.Length;
              return Handler('text');
            end function;
            public function Plain(): string;
            begin return new N.Plain.Inner<string>('plain').Value; end function;
            public function Related(): N.Outer<object>.Related<string>;
            begin return new N.Outer<object>.Related<string>(); end function;
            public function Echo(): string;
            begin return S.Nesting.Outer<string>.Inner<int>.Create('value', 7).Echo<string>('echo'); end function;
            public function Inspect(): Sys.Type;
            begin return typeof(N.Outer<string?>.Inner<int>.Deep<string>); end function;
            public function Optional(Value: N.Outer<string?>.Inner<int>?[]?): N.Outer<string?>.Inner<int>?[]?;
            begin return Value; end function;
            public function Validate(Value: object): bool;
            begin
              const Cast: Selected := cast<Selected>(Value);
              return Value is N.Outer<string?>.Inner<int> and Cast.Second = 42;
            end function;
            public function Keys(): string;
            begin
              const Values: C.Dictionary<string, int> := new C.Dictionary<string, int>();
              Values.Add('key', 42);
              const Items: C.Dictionary<string, int>.KeyCollection := Values.Keys;
              var Iterator: C.Dictionary<string, int>.KeyCollection.Enumerator := Items.GetEnumerator();
              if Iterator.MoveNext() then return Iterator.Current; else return 'missing'; end if;
            end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Consumer"));
        File.WriteAllText(project.File("Consumer/Consumer.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Consumer/Consumer.fpas"), """
            unit Demo.Consumer;
            uses Demo.Nested as N, Shared.Nesting as S;
            public function Replay(): S.Outer<string?>.Inner<int>;
            begin
              var Value: N.Selected := N.Select();
              return N.Exchange<string?, int>(ref Value);
            end function;
            end unit;
            """);
        var host = XDocument.Load(project.File("Host/Host.csproj"));
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj"))));
        host.Save(project.File("Host/Host.csproj"));
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using Shared.Nesting;
            using System.Reflection;
            Outer<string?>.Inner<int> value = Nested.Select();
            if (value.First is not null || value.Second != 42 || !Nested.Validate(value)) throw new Exception("Nested owner arguments");
            if (Nested.Create("created", 7).First != "created" || Nested.Echo() != "echo") throw new Exception("Constructors and methods");
            if (Consumer.Replay().Second != 42) throw new Exception("Exported alias and reference reader");
            if (Nested.Leaf(null).Value is not null || Nested.Cell(null).Value is not null) throw new Exception("Inherited arguments");
            var deep = Nested.Deep<object, int, string>("deep");
            if (deep.Value != "deep" || deep.GetType() != typeof(Outer<object>.Inner<int>.Deep<string>) || Nested.Inspect() != typeof(Outer<string>.Inner<int>.Deep<string>)) throw new Exception("Three levels and type tokens");
            if (Nested.Related().GetType() != typeof(Outer<object>.Related<string>)) throw new Exception("Constraint referencing an outer argument");
            if (Nested.Map() != 4 || Nested.Plain() != "plain" || Nested.Keys() != "key") throw new Exception("Nested delegate, static container, BCL struct");
            Outer<string?>.Inner<int>?[] items = [value, null];
            if (!ReferenceEquals(items, Nested.Optional(items)) || Nested.Optional(null) is not null) throw new Exception("Nullable arrays");
            var info = new NullabilityInfoContext().Create(typeof(Nested).GetMethod("Optional")!.ReturnParameter);
            if (info.ReadState != NullabilityState.Nullable || info.ElementType!.ReadState != NullabilityState.Nullable ||
                info.ElementType.GenericTypeArguments[0].ReadState != NullabilityState.Nullable) throw new Exception("Nested nullable metadata");
            Console.WriteLine("Nested CLR types verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Nested CLR types verified" + Environment.NewLine, run.Output);
        var expected = XDocument.Load(project.File("Shared/bin/Debug/net10.0/Shared.xml")).Descendants("member")
            .Single(member => ((string)member.Attribute("name")!).StartsWith("M:Shared.Nesting.Oracle.Exchange", StringComparison.Ordinal));
        var actual = XDocument.Load(project.File("Game/bin/Debug/net10.0/Game.xml")).Descendants("member")
            .Single(member => ((string)member.Attribute("name")!).StartsWith("M:Demo.Nested.Exchange", StringComparison.Ordinal));
        Assert.Equal(((string)expected.Attribute("name")!).Replace("Shared.Nesting.Oracle", "Demo.Nested", StringComparison.Ordinal), (string)actual.Attribute("name")!);
    }

    [Fact]
    public async Task Invalid_nested_type_paths_report_the_failing_segment_before_emission()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Nested.cs"), NestedTypeReference);
        var build = await sdk.Run(project, "build", "Shared/Shared.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        (string Path, string Segment, string Code)[] cases =
        [
            ("N.Outer<int>.Inner<int>", "Outer", "FP3018"),
            ("N.Outer<string>.Inner<string>", "Inner", "FP3018"),
            ("N.Outer<string>.Related<object>", "Related", "FP3018"),
            ("N.Outer<string>.Inner<int, int>", "Inner", "FP3001"),
            ("N.Outer<string>.Leaf<int>", "Leaf", "FP3001"),
            ("N.Outer<string>.Hidden", "Hidden", "FP3001"),
            ("N.Outer<string>.Missing<int>", "Missing", "FP3001"),
            ("N.Outer<string>.Inner", "Inner", "FP3001"),
            ("N<int>.Plain", "N", "FP3018")
        ];
        foreach (var (path, segment, code) in cases)
        {
            var text = "program Invalid; uses Shared.Nesting as N; type Target = " + path + "; begin end; end program;";
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Invalid.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"], text);
            Assert.NotEqual(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            var diagnostic = Assert.Single(document.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.True(code == diagnostic.GetProperty("code").GetString(), path + ": " + diagnostic.GetProperty("message").GetString());
            var column = text.IndexOf(path, StringComparison.Ordinal) + path.LastIndexOf(segment, StringComparison.Ordinal) + 1;
            Assert.Equal(column, diagnostic.GetProperty("location").GetProperty("column").GetInt32());
            Assert.False(File.Exists(project.File("Invalid.dll")));
        }
    }
}
