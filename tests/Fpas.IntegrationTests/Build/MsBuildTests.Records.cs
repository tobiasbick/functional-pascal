using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Inventory_model_example_builds_and_preserves_the_original_positions()
    {
        using var project = sdk.CreateProject();
        Directory.CreateDirectory(project.File("Inventory"));
        var fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "inventory-model");
        foreach (var source in Directory.GetFiles(fixture))
            File.WriteAllText(project.File("Inventory/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Inventory/InventoryModel.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Inventory/bin/Debug/net10.0/InventoryModel.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "Before: 3", "After: 2", "Weight: 2", "Original unchanged: True", "Item retained: True", ""), run.Output);
    }

    [Fact]
    public async Task Records_defaults_and_updates_cross_FPAS_assemblies_and_are_consumable_from_CSharp()
    {
        using var project = sdk.CreateProject();
        Directory.CreateDirectory(project.File("Models"));
        File.WriteAllText(project.File("Models/Models.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>
            """);
        File.WriteAllText(project.File("Models/Models.fpas"), """
            unit Demo.Models;
            uses System.Console as Console;
            function DefaultLabel(): string;
            begin Console.WriteLine('default'); return 'initial'; end function;
            public type Item = record
              public Label: string := DefaultLabel();
              public Count: int := 1;
              public Note: string? := null;
              public function Increase(): Item;
              begin return self with Count := self.Count + 1; end with; end function;
            end record;
            type State = record public Value: int; end record;
            public type Counter = record
              State: State;
              public Label: string := 'counter';
              static function ReadState(Value: State): int;
              begin return Value.Value; end function;
              public static function Create(Value: int): Counter;
              begin return new Counter(new State(Value)); end function;
              public function Current(): int;
              begin return ReadState(self.State); end function;
            end record;
            end unit;
            """);
        var gameProject = project.File("Game/Game.fpasproj");
        File.WriteAllText(gameProject, File.ReadAllText(gameProject).Replace("</ItemGroup>", "<ProjectReference Include=\"../Models/Models.fpasproj\" /></ItemGroup>", StringComparison.Ordinal));
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses Demo.Models as M;
            public function Make(): M.Item;
            begin return new M.Item(Count := 2); end function;
            public function Rename(Value: M.Item): M.Item;
            begin return Value with Label := 'FPAS'; Count := Value.Count + 1; end with; end function;
            public function RenameCounter(): M.Counter;
            begin return M.Counter.Create(42) with Label := 'changed'; end with; end function;
            public function HasNote(Value: M.Item): bool;
            begin return Value.Note <> null; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using System.Reflection;
            var original = Bridge.Make();
            if (original.Label != "initial" || original.Count != 2 || original.Note != null) throw new Exception("Defaults");
            var updated = Bridge.Rename(original);
            var expected = new Models.Item("FPAS", 3, null);
            if (original.Label != "initial" || updated.Label != "FPAS") throw new Exception("Immutability");
            if (updated != expected || !updated.Equals((object)expected) || !((IEquatable<Models.Item>)updated).Equals(expected)) throw new Exception("Equality");
            if (updated.GetHashCode() != expected.GetHashCode() || new HashSet<Models.Item> { updated, expected }.Count != 1) throw new Exception("Hashing");
            Models.Item? missing = null;
            if (missing != null || updated == null || updated.Equals((object?)null)) throw new Exception("Null equality");
            if (Bridge.HasNote(expected) || !Bridge.HasNote(new Models.Item("C#", 4, "note"))) throw new Exception("Nullable fields");
            if (updated.Increase().Count != 4) throw new Exception("Instance method");
            var counter = Bridge.RenameCounter();
            var counterType = typeof(Models.Counter);
            if (counter.Current() != 42 || counter.Label != "changed" || counterType.GetConstructors().Length != 0 ||
                counterType.GetProperty("State") is not null || counterType.GetMethod("ReadState") is not null ||
                counterType.GetProperty("State", BindingFlags.Instance | BindingFlags.NonPublic)?.GetMethod?.IsPrivate != true ||
                counterType.GetMethod("ReadState", BindingFlags.Static | BindingFlags.NonPublic)?.IsPrivate != true)
                throw new Exception("Private state or helper visibility");
            if (!typeof(Models.Item).IsSealed || typeof(Models.Item).GetProperty("Label")!.CanWrite) throw new Exception("CLR shape");
            var marker = typeof(Models.Item).GetCustomAttributes(false).Single(attribute => attribute.GetType().FullName == "Fpas.Core.RecordAttribute");
            if ((int)marker.GetType().GetProperty("Version")!.GetValue(marker)! != 1) throw new Exception("Record marker");
            var fields = typeof(Models.Item).GetProperties().Select(property => property.GetCustomAttributes(false).Single(attribute => attribute.GetType().FullName == "Fpas.Core.RecordFieldAttribute")).ToArray();
            if (fields.Length != 3 || fields.Select(field => (int)field.GetType().GetProperty("Order")!.GetValue(field)!).Distinct().Count() != 3) throw new Exception("Field metadata");
            Console.WriteLine("CLR records verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("default" + Environment.NewLine + "CLR records verified" + Environment.NewLine, run.Output);
    }
}
