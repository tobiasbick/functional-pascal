using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Generic_tree_example_builds_and_runs()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Tree"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "generic-tree")))
            File.WriteAllText(project.File("Tree/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Tree/GenericTree.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Tree/bin/Debug/net10.0/GenericTree.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "(21, 34)", "leaves: 2", "True", ""), run.Output);
    }

    [Fact]
    public async Task Generic_records_and_variants_preserve_their_contract_across_CSharp_and_Fpas_assemblies()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Models.fpas"), """
            unit Demo.Models;
            uses System as Sys;
            public type Box<T> = record
              public Value: T;
              public Label: string := 'default';
              public function Get(): T; begin return self.Value; end function;
              public function Capture<U>(Other: U): function(): function(): U;
              begin return function(): function(): U => function(): U begin discard self.Value; return Other; end function; end function;
              public function Upcast<U>(Other: U): T; where U: T; begin return Other; end function;
            end record;
            public type ValueBox<T> = where T: struct;
            record public Value: T?; public Label: string? := null; end record;
            public type Factory<T> = where T: class, new();
            record
              public Value: T := new T();
              public function Read(): function(): T; begin return function(): T => self.Value; end function;
            end record;
            public type Tree<T> = variant Empty; Leaf(Value: T); Branch(Left: Tree<T>; Right: Tree<T>); end variant;
            public type Pair<T> = where T: Sys.IComparable<T>;
            record
              public Left: T; public Right: T;
              public function Compare(): int; begin return self.Left.CompareTo(self.Right); end function;
              public function Read(): function(): int; begin return function(): int => self.Compare(); end function;
            end record;
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
            uses Demo.Models as M, System as Sys;
            public function Box<T>(Value: T): M.Box<T>; begin return new M.Box<T>(Value); end function;
            public function Copy<T>(Value: M.Box<T>; Next: T): M.Box<T>; begin return Value with Value := Next; end with; end function;
            public function Missing<T>(Value: T?): M.ValueBox<T>; where T: struct;
            begin return new M.ValueBox<T>(Value); end function;
            public function Compare<T>(Left: T; Right: T): int; where T: Sys.IComparable<T>;
            begin return new M.Pair<T>(Left, Right).Read()(); end function;
            public function Create<T>(): M.Factory<T>; where T: class, new();
            begin return new M.Factory<T>(); end function;
            public function Build<T>(Value: T): M.Tree<T>;
            begin return M.Tree<T>.Branch(M.Tree<T>.Leaf(Value), M.Tree<T>.Empty); end function;
            public function Count<T>(Value: M.Tree<T>): int;
            begin return case Value of
              when M.Tree.Empty: 0;
              when M.Tree.Leaf(const Item: T): 1;
              when M.Tree.Branch(const Left: M.Tree<T>, const Right: M.Tree<T>): Count<T>(Left) + Count<T>(Right);
            end case; end function;
            end unit;
            """);
        var hostFile = project.File("Host/Host.csproj"); var host = XDocument.Load(hostFile);
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj")))); host.Save(hostFile);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using System.Reflection;
            var box = Consumer.Box(42); var copy = Consumer.Copy(box, 7);
            if (box.Get() != 42 || box.Label != "default" || copy.Value != 7 || copy.Label != "default") throw new Exception("Record defaults/copy");
            if (box.Capture("saved")()() != "saved" || box.Upcast(3) != 3) throw new Exception("Type/method parameters");
            if (box != new Models.Box<int>(42, "default") || box.Equals(copy)) throw new Exception("Typed equality");
            var missing = Consumer.Missing<int>(null);
            if (missing.Value != null || missing.Label != null || Consumer.Missing<int>(42).Value != 42) throw new Exception("Nullable value metadata");
            if (Consumer.Compare(1, 2) >= 0 || new Models.Pair<string>("a", "b").Read()() >= 0) throw new Exception("Constraints in closures");
            var tree = Consumer.Build("text");
            if (Consumer.Count(tree) != 1 || tree is not Models.Tree<string>.BranchCase branch || branch.Left is not Models.Tree<string>.LeafCase { Value: "text" }) throw new Exception("Variant payload/matching");
            if (Models.Tree<int>.Leaf(42) != Models.Tree<int>.Leaf(42) || Models.Tree<int>.Empty == Models.Tree<int>.Leaf(42)) throw new Exception("Variant equality");
            if (Consumer.Count(Models.Tree<int>.Empty) != 0) throw new Exception("Fieldless generic case");
            if (Consumer.Create<List<int>>().Read()().Count != 0) throw new Exception("Constructor constraint in default/closure");
            if (!typeof(Models.Box<>).IsGenericTypeDefinition || typeof(Models.Tree<>.LeafCase).GetGenericArguments().Length != 1) throw new Exception("CLR type definitions");
            var constraint = typeof(Models.Pair<>).GetGenericArguments()[0].GetGenericParameterConstraints().Single();
            if (constraint.GetGenericTypeDefinition() != typeof(IComparable<>)) throw new Exception("Type constraints");
            Console.WriteLine("Generic data verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Generic data verified" + Environment.NewLine, run.Output);
        File.WriteAllText(project.File("Host/Invalid.cs"), """
            public static class Invalid { public static string Read() => Demo.Consumer.Box<string?>(null).Value; }
            """);
        var nullable = await sdk.Run(project, "build", "Host/Host.csproj", "--no-restore", "-p:TreatWarningsAsErrors=true");
        Assert.NotEqual(0, nullable.ExitCode); Assert.Contains("CS8603", nullable.Output);
    }
}
