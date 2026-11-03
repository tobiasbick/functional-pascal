using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Inference_handles_imported_overloads_constraints_variance_and_FPAS_union_metadata()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/InferenceApi.cs"), """
            namespace Demo;
            public static class InferenceApi
            {
                public static string Kind<T>(T value) => typeof(T).Name;
                public static string Choose<T>(T left, T right) => typeof(T).Name;
                public static string Pick(object value) => "object";
                public static string Pick(int value) => "int";
                public static string Pick<T>(T value) => "generic";
                public static string Specific<T>(T value) => "value";
                public static string Specific<T>(T[] value) => "array";
                public static string Output<T>(Func<T> read) => Kind(read());
                public static string Custom<T>(Comparison<T> compare, T value) => typeof(T).Name;
                public static string Nullable<T>(T? value) where T : struct => typeof(T).Name;
                public static string Ref<T>(T value) where T : class => typeof(T).Name;
                public static string Variance<T>(IEnumerable<T> values, IComparer<T> comparer) => typeof(T).Name;
                public static string Ambiguous<T>(IComparable<T> value) => "compare";
                public static string Ambiguous<T>(IEquatable<T> value) => "equal";
                public static string View<T>(IEnumerable<T> value) => typeof(T).Name;
            }
            public sealed class ObjectComparer : IComparer<object> { public int Compare(object? x, object? y) => 0; }
            public sealed class Collision<T> { }
            public sealed class Collision<T, U> { }
            public sealed class Dual : IEnumerable<int>, IEnumerable<string>
            {
                IEnumerator<int> IEnumerable<int>.GetEnumerator() => throw new NotSupportedException();
                IEnumerator<string> IEnumerable<string>.GetEnumerator() => throw new NotSupportedException();
                System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator() => throw new NotSupportedException();
            }
            """);
        File.WriteAllText(project.File("Game/InferenceLibrary.fpas"), """
            unit Demo.InferenceLibrary;
            public type Tree<T> = union Empty; Leaf(Value: T); Branch(Left: Tree<T>; Right: Tree<T>); end union;
            public type RefChoice<T> = where T: class; union Empty; Value(Item: T); end union;
            public type MaybeValue<T> = where T: struct; union Empty; Value(Item: T?); end union;
            public function Identity<T>(Value: T): T; begin return Value; end function;
            public function Count<T>(Value: Tree<T>): int;
            begin return case Value of when Tree.Empty: 0; when Tree.Leaf(_): 1; when Tree.Branch(const L: Tree<T>, const R: Tree<T>): Count(L) + Count(R); end case; end function;
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
            uses Demo.InferenceLibrary as L, Demo.InferenceApi as Api, Demo as D;
            public function Run(): string[];
            begin
              const Tree: L.Tree<int> := L.Tree.Branch(L.Tree.Empty, L.Tree.Leaf(L.Identity(42)));
              const Empty: L.Tree<string> := L.Tree.Empty;
              const Maybe: int? := 9;
              const Optional: L.MaybeValue<int> := L.MaybeValue.Value(null);
              return [Api.Kind(L.Count(Tree)), Api.Kind(Empty), Api.Choose(1, 2.5), Api.Pick(1), Api.Pick('text'), Api.Specific([1, 2]),
                Api.Output(function(): int => 3), Api.Custom(function(Left: int; Right: int): int => Left - Right, 1), Api.Nullable(Maybe),
                Api.Variance(['text'], new D.ObjectComparer()), Api.Kind(new D.Dual()),
                case Optional of when L.MaybeValue.Empty: 'empty'; when L.MaybeValue.Value(const Item: int?): if Item = null then 'null' else 'value' end if; end case,
                Api.Kind(L.RefChoice.Value('text'))];
            end function;
            end unit;
            """);
        var hostFile = project.File("Host/Host.csproj"); var host = XDocument.Load(hostFile);
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj")))); host.Save(hostFile);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            var expected = new[] { InferenceApi.Kind(InferenceLibrary.Count(InferenceLibrary.Tree<int>.Leaf(42))), "Tree`1",
                InferenceApi.Choose(1, 2.5), InferenceApi.Pick(1), InferenceApi.Pick("text"), InferenceApi.Specific(new[] { 1, 2 }),
                InferenceApi.Output(() => 3), InferenceApi.Custom((int left, int right) => left - right, 1), InferenceApi.Nullable((int?)9),
                InferenceApi.Variance(new[] { "text" }, new ObjectComparer()), "Dual", "null", "RefChoice`1" };
            if (!Consumer.Run().SequenceEqual(expected)) throw new Exception("FPAS: " + string.Join(",", Consumer.Run()) + "; C#: " + string.Join(",", expected));
            Console.WriteLine("Inference matches C#");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Inference matches C#" + Environment.NewLine, run.Output);
        foreach (var (expression, code) in new[] { ("Api.Ref(1)", "FP3018"), ("Api.Ambiguous(1)", "FP3003"), ("Api.View(new D.Dual())", "FP3026"), ("L.RefChoice.Value(1)", "FP3018"), ("D.Collision.Make(1)", "FP4002") })
        {
            File.WriteAllText(project.File("Consumer/Invalid.fpas"), "unit Demo.Invalid; uses Demo.InferenceApi as Api, Demo.InferenceLibrary as L, Demo as D; public procedure Test(); begin discard " + expression + "; end procedure; end unit;");
            var invalid = await sdk.Run(project, "build", "Consumer/Consumer.fpasproj", "--no-restore");
            Assert.NotEqual(0, invalid.ExitCode); Assert.Contains(code, invalid.Output + invalid.Error);
        }
    }
}
