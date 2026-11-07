using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class RenameGraphTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Cross_DLL_calls_and_named_arguments_are_planned_against_temporary_metadata_and_execute_after_application()
    {
        const string library = "unit Demo.Bridge; public function Read(Value: int): int; begin return Value + 1; end function; end unit;";
        const string consumer = "program Consumer; uses Demo.Bridge as B, System.Console as Console; begin Console.WriteLine(B.Read(Value := 41)); end; end program;";
        using var directory = RenameFixture.Create(sdk, library);
        Directory.CreateDirectory(directory.File("Consumer")); File.WriteAllText(directory.File("Consumer/Main.fpas"), consumer);
        File.WriteAllText(directory.File("Consumer/Consumer.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup><ItemGroup><ProjectReference Include=\"../Game/Game.fpasproj\" /></ItemGroup></Project>");
        var built = await sdk.Run(directory, "build", "Consumer/Consumer.fpasproj", "-m:1", "--disable-build-servers"); Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var assembly = directory.File("Consumer/bin/Debug/net10.0/Consumer.dll");
        var original = await ProcessRunner.RunAsync([assembly]); Assert.Equal(0, original.ExitCode); Assert.Equal("42", original.Output.Trim());
        var analysis = await RenameFixture.Analyze(directory, "Consumer/Consumer.fpasproj");
        var selected = RenameFixture.Select(directory, consumer, "Read(Value", "Consumer/Main.fpas");
        var prepared = RenameService.Prepare(analysis, selected, "Fetch");
        Assert.Equal(2, prepared.Plan.EditCount); Assert.Equal(2, prepared.Plan.Files.Count);
        Assert.Equal(library, File.ReadAllText(directory.File("Game/Bridge.fpas"))); Assert.Equal(consumer, File.ReadAllText(directory.File("Consumer/Main.fpas")));
        await SourcePublication.ApplyAsync(prepared.Edits.ToSourceChanges(), analysis.Context.VerifyInputsAsync);
        var rebuilt = await sdk.Run(directory, "build", "Consumer/Consumer.fpasproj", "--no-restore", "-m:1", "--disable-build-servers"); Assert.True(rebuilt.ExitCode == 0, rebuilt.Output + rebuilt.Error);
        var renamed = await ProcessRunner.RunAsync([assembly]); Assert.Equal(0, renamed.ExitCode); Assert.Equal(original.Output, renamed.Output);
        analysis = await RenameFixture.Analyze(directory, "Consumer/Consumer.fpasproj");
        var updatedLibrary = File.ReadAllText(directory.File("Game/Bridge.fpas"));
        var parameter = RenameService.Prepare(analysis, RenameFixture.Select(directory, updatedLibrary, "Value: int"), "Input");
        Assert.Contains(parameter.Plan.Files.SelectMany(file => file.Edits), edit => edit.Location.Path == directory.File("Consumer/Main.fpas") && edit.ExpectedText == "Value");
    }

    [Theory]
    [InlineData("Item<T> =", "Item", "Node")]
    [InlineData("T> = record", "T", "U")]
    [InlineData("Alias;", "Alias", "UnitAlias")]
    [InlineData("Bridge;", "Bridge", "Renamed")]
    public async Task Type_parameters_aliases_and_unit_names_are_checked(string marker, string oldName, string newName)
    {
        const string source = """
            unit Demo.Bridge;
            uses System as Alias;
            public type Item<T> = record public Value: T; end record;
            public function Run(): int;
            begin const X: Item<int> := new Item<int>(Value := 42); return Alias.Math.Abs(X.Value); end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source); var analysis = await RenameFixture.Analyze(directory);
        var prepared = RenameService.Prepare(analysis, RenameFixture.Select(directory, source, marker), newName);
        Assert.NotEmpty(prepared.Plan.Files); Assert.All(prepared.Plan.Files.SelectMany(file => file.Edits), edit => Assert.Equal(oldName, edit.ExpectedText));
        Assert.Contains(newName, Assert.Single(prepared.Edits.Documents).NewText);
    }

    [Fact]
    public async Task Unit_renames_update_qualified_type_import_prefixes()
    {
        const string library = "unit Demo.Bridge; public type Point = record public X: int; end record; end unit;";
        const string consumer = "unit Consumer; uses Demo.Bridge.Point as P; public function Read(): int; begin return new P(X := 42).X; end function; end unit;";
        using var directory = RenameFixture.Create(sdk, library); File.WriteAllText(directory.File("Game/Consumer.fpas"), consumer);
        var analysis = await RenameFixture.Analyze(directory);
        var prepared = RenameService.Prepare(analysis, RenameFixture.Select(directory, library, "Bridge;"), "Geometry");
        Assert.Equal(2, prepared.Plan.Files.Count); Assert.Equal(2, prepared.Plan.EditCount);
        Assert.Contains(prepared.Edits.Documents, document => document.NewText.Contains("uses Demo.Geometry.Point as P", StringComparison.Ordinal));
    }

    [Fact]
    public async Task Linked_sources_require_an_owner_and_shared_declarations_are_renamed_in_every_owning_project()
    {
        const string source = "unit Shared; public function Read(): int; begin return 42; end function; end unit;";
        using var directory = RenameFixture.Create(sdk, source);
        Directory.CreateDirectory(directory.File("Other"));
        File.WriteAllText(directory.File("Other/Other.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType><EnableDefaultFpasCompileItems>false</EnableDefaultFpasCompileItems></PropertyGroup><ItemGroup><Compile Include=\"../Game/Bridge.fpas\" /></ItemGroup></Project>");
        var solution = System.Xml.Linq.XDocument.Load(directory.File("Game.slnx"));
        solution.Root!.Add(new System.Xml.Linq.XElement("Project", new System.Xml.Linq.XAttribute("Path", "Other/Other.fpasproj"))); solution.Save(directory.File("Game.slnx"));
        var analysis = await RenameFixture.Analyze(directory, "Game.slnx"); var selection = RenameFixture.Select(directory, source, "Read():");
        Assert.Equal("ambiguous_project", Assert.Throws<ContextFailure>(() => RenameService.Prepare(analysis, selection, "Fetch")).Code);
        var prepared = RenameService.Prepare(analysis, selection with { Project = directory.File("Game/Game.fpasproj") }, "Fetch");
        Assert.Equal(2, prepared.Plan.Targets.Count); Assert.Single(prepared.Plan.Files); Assert.Equal(1, prepared.Plan.EditCount);
    }
}
