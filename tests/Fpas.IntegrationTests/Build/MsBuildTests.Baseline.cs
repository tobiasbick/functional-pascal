using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Mixed_solution_restores_builds_and_runs_FSharp_Fpas_and_CSharp_with_transitive_assets(string configuration)
    {
        using var project = sdk.CreateProject();
        var properties = XDocument.Load(project.File("Directory.Build.props"));
        properties.Root!.Element("PropertyGroup")!.Add(new XElement("RestoreAdditionalProjectSources", "$(MSBuildSDKsPath)/../FSharp/library-packs"),
            new XElement("RestorePackagesPath", project.File("packages")), new XElement("NuGetAudit", "false"));
        properties.Save(project.File("Directory.Build.props"));
        Directory.CreateDirectory(project.File("FSharpShared"));
        Directory.CreateDirectory(project.File("FSharpHost"));
        File.WriteAllText(project.File("FSharpShared/FSharpShared.fsproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><Compile Include="Library.fs" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("FSharpShared/Library.fs"), """
            namespace CrossLanguage
            type Api =
                static member Double(value: int) = List.sum [value; value]
                static member Step = 2
            """);
        File.WriteAllText(project.File("FSharpHost/FSharpHost.fsproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><OutputType>Exe</OutputType></PropertyGroup>
              <ItemGroup>
                <Compile Include="Program.fs" />
                <ProjectReference Include="../Game/Game.fpasproj" />
              </ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("FSharpHost/Program.fs"), """
            module CrossLanguage.Host
            open Demo
            [<EntryPoint>]
            let main _ =
                let pair = Bridge.Build()
                if pair.Left <> 40 || pair.Right <> 2 || Bridge.Answer() <> 42 then
                    failwith "Incorrect FPAS metadata or F# dependency behavior"
                printfn "F# interop verified"
                0
            """);
        var game = XDocument.Load(project.File("Game/Game.fpasproj"));
        game.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../FSharpShared/FSharpShared.fsproj"))));
        game.Save(project.File("Game/Game.fpasproj"));
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses CrossLanguage as F;
            public type Pair = record public Left: int; public Right: int; end record;
            public function Build(): Pair;
            begin return new Pair(F.Api.Double(20), F.Api.Step); end function;
            public function Answer(): int;
            begin const Value: Pair := Build(); return Value.Left + Value.Right; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            if (Demo.Bridge.Answer() != 42) throw new Exception("Incorrect F# to FPAS to C# behavior");
            Console.WriteLine("C# interop verified");
            """);
        var solution = XDocument.Load(project.File("Game.slnx"));
        solution.Root!.Add(new XElement("Project", new XAttribute("Path", "FSharpShared/FSharpShared.fsproj")),
            new XElement("Project", new XAttribute("Path", "FSharpHost/FSharpHost.fsproj")));
        solution.Save(project.File("Game.slnx"));
        var build = await sdk.Run(project, "build", "Game.slnx", "--configuration", configuration, "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        foreach (var (host, expected) in new[] { ("Host", "C# interop verified"), ("FSharpHost", "F# interop verified") })
        {
            var output = $"{host}/bin/{configuration}/net10.0/";
            var run = await sdk.Run(project, output + host + ".dll");
            Assert.True(run.ExitCode == 0, run.Output + run.Error);
            Assert.Equal(expected + Environment.NewLine, run.Output);
            foreach (var asset in new[] { "Game.dll", "FSharpShared.dll", "FSharp.Core.dll", "Fpas.Core.dll" })
                Assert.True(File.Exists(project.File(output + asset)), asset);
        }
    }

    [Fact]
    public async Task Record_interfaces_are_known_before_validating_recursive_generic_fields_and_signatures()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Graph;
            uses System as Sys;
            public type Box<T> = where T: Sys.IComparable<T>;
            record public Value: T; end record;
            public type ItemBox = Box<Item>;
            public type Item = record implements Sys.IComparable<Item>
              public Value: int;
              public Children: array of ItemBox := [];
              public function CompareTo(Other: Item?): int;
              begin
                if Other = null then return 1; end if;
                return self.Value.CompareTo(Other.Value);
              end function;
              public function Wrap(): ItemBox;
              begin return new ItemBox(self); end function;
            end record;
            public type Link<T> = where T: Sys.IComparable<T>;
            variant Empty; Next(Value: Box<T>); end variant;
            public function Build(): ItemBox;
            begin
              const Entry: Item := new Item(42);
              const Tree: Link<Item> := Link.Next(Entry.Wrap());
              return case Tree of
                when Link.Empty: Entry.Wrap();
                when Link.Next(const Value: ItemBox): Value;
              end case;
            end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            var box = Graph.Build();
            IComparable<Graph.Item> comparable = box.Value;
            if (box.Value.Value != 42 || box.Value.Children.Length != 0 || comparable.CompareTo(null) != 1 ||
                comparable.CompareTo(new Graph.Item(41, [])) != 1 || box.Value.Wrap().Value != box.Value)
                throw new Exception("Incomplete declaration graph or invalid CLR interface signature");
            Console.WriteLine("Declaration graph verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Declaration graph verified" + Environment.NewLine, run.Output);
    }
}
