using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Game_action_example_builds_and_matches_each_domain_outcome()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Actions"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "game-actions")))
            File.WriteAllText(project.File("Actions/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Actions/GameActions.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Actions/bin/Debug/net10.0/GameActions.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "Moved to 1,0", "Blocked: wall", "Found key at 2,0", "Stayed", ""), run.Output);
    }

    [Fact]
    public async Task Unions_are_consumable_from_CSharp_and_matched_by_a_second_FPAS_project()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Models"));
        File.WriteAllText(project.File("Models/Models.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>
            """);
        File.WriteAllText(project.File("Models/Models.fpas"), """
            unit Demo.Models;
            public type Outcome = union
              Wait;
              Move(X: int; Y: int);
              Block(Reason: string?);
              Pair(First: Outcome; Second: Outcome);
              get_Data(Value: int);
            end union;
            end unit;
            """);
        var path = project.File("Game/Game.fpasproj"); var xml = XDocument.Load(path);
        xml.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Models/Models.fpasproj")))); xml.Save(path);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses Demo.Models as M;
            public function Make(): M.Outcome;
            begin return M.Outcome.Move(Y := 2, X := 1); end function;
            public function Describe(Value: M.Outcome): string;
            begin
              return case Value of
                when M.Outcome.Wait: 'wait';
                when M.Outcome.Move(Y := const Y: int, X := const X: int): X.ToString() + ',' + Y.ToString();
                when M.Outcome.Block(const Reason: string?): Reason ?? 'unknown';
                when M.Outcome.Pair(M.Outcome.Wait, const Other: M.Outcome): Describe(Other);
                when M.Outcome.Pair(_, _): 'pair';
                when M.Outcome.get_Data(const N: int): N.ToString();
              end case;
            end function;
            public function IsMissing(Value: M.Outcome?): bool;
            begin return case Value of when null: true; when const Present: M.Outcome?: false; end case; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using System.Reflection;
            var first = Bridge.Make(); var equal = Models.Outcome.Move(1, 2);
            if (first is not Models.Outcome.MoveCase moved || moved.X != 1 || moved.Y != 2) throw new Exception("Case shape");
            if (first != equal || !first.Equals((object)equal) || !((IEquatable<Models.Outcome>)first).Equals(equal)) throw new Exception("Value equality");
            if (first.GetHashCode() != equal.GetHashCode() || new HashSet<Models.Outcome> { first, equal }.Count != 1) throw new Exception("Hashing");
            if (Bridge.Describe(first) != "1,2" || Bridge.Describe(Models.Outcome.Block(null)) != "unknown") throw new Exception("Payloads");
            if (Bridge.Describe(Models.Outcome.Pair(Models.Outcome.Wait, first)) != "1,2") throw new Exception("Nested matching");
            if (!Bridge.IsMissing(null) || Bridge.IsMissing(first) || first == null || Models.Outcome.Wait != Models.Outcome.Wait) throw new Exception("Null/fieldless equality");
            if (Bridge.Describe(Models.Outcome.get_Data(7)) != "7" || typeof(Models.Outcome).GetMethod("get_Data")!.IsSpecialName) throw new Exception("Factory naming");
            var root = typeof(Models.Outcome); var concrete = typeof(Models.Outcome.MoveCase);
            if (!root.IsAbstract || !concrete.IsSealed || concrete.BaseType != root || root.GetProperty("Tag")!.CanWrite) throw new Exception("Union representation");
            if (!root.GetConstructors(BindingFlags.Instance | BindingFlags.NonPublic).Single().IsFamilyAndAssembly || concrete.GetConstructors().Length != 0) throw new Exception("Closed construction");
            var marker = root.GetCustomAttributes(false).Single(attribute => attribute.GetType().FullName == "Fpas.Core.UnionAttribute");
            if ((int)marker.GetType().GetProperty("Version")!.GetValue(marker)! != 1) throw new Exception("Marker version");
            var member = concrete.GetCustomAttributes(false).Single(attribute => attribute.GetType().FullName == "Fpas.Core.UnionCaseAttribute");
            if (!((string[])member.GetType().GetProperty("PayloadNames")!.GetValue(member)!).SequenceEqual(new[] { "X", "Y" })) throw new Exception("Payload metadata");
            Console.WriteLine("CLR unions verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error); Assert.Equal("CLR unions verified" + Environment.NewLine, run.Output);
        File.WriteAllText(project.File("Host/Rogue.cs"), """
            public sealed class Rogue : Demo.Models.Outcome
            {
                public override int Tag => 99;
                public override bool Equals(object? other) => ReferenceEquals(this, other);
                public override int GetHashCode() => 99;
            }
            """);
        var inheritance = await sdk.Run(project, "build", "Host/Host.csproj", "--no-restore");
        Assert.NotEqual(0, inheritance.ExitCode); Assert.Contains("CS0122", inheritance.Output);
    }
}
