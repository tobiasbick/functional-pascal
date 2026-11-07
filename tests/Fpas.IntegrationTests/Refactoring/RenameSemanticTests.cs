using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class RenameSemanticTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Rename_planning_uses_the_captured_snapshot_without_reading_changed_source_files()
    {
        const string source = "unit Demo.Bridge; public function Read(): int; begin return 42; end function; end unit;";
        using var directory = RenameFixture.Create(sdk, source); var analysis = await RenameFixture.Analyze(directory);
        File.WriteAllText(directory.File("Game/Bridge.fpas"), "unsaved disk contents differ from the captured source");
        var prepared = RenameService.Prepare(analysis, RenameFixture.Select(directory, source, "Read():"), "Fetch");
        Assert.Equal(source.Replace("Read", "Fetch", StringComparison.Ordinal), Assert.Single(prepared.Edits.Documents).NewText);
        Assert.Equal("inputs_changed", (await Assert.ThrowsAsync<ContextFailure>(() => analysis.Context.VerifyInputsAsync())).Code);
        Assert.Equal("unsaved disk contents differ from the captured source", File.ReadAllText(directory.File("Game/Bridge.fpas")));
    }

    [Fact]
    public async Task One_overload_and_its_callable_references_are_renamed_without_touching_comments_strings_or_other_overloads()
    {
        const string source = """
            unit Demo.Bridge;
            // Transform stays in documentation.
            function Transform(Value: int): int; begin return Value + 1; end function;
            function Transform(Value: string): int; begin return Value.Length; end function;
            public function Run(): int;
            begin
              const F: function(int): int := Transform;
              const Label: string := 'Transform';
              return F(Transform(Value := 40)) + Transform(Label);
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source); var analysis = await RenameFixture.Analyze(directory);
        var prepared = RenameService.Prepare(analysis, RenameFixture.Select(directory, source, "Transform(Value: int)"), "Map");
        Assert.Equal(3, prepared.Plan.EditCount); Assert.Single(prepared.Plan.Targets); Assert.False(prepared.Plan.ChangesPublicApi);
        Assert.Equal(source, File.ReadAllText(directory.File("Game/Bridge.fpas")));
        var replacement = Assert.Single(prepared.Edits.Documents).NewText;
        Assert.Contains("function Map(Value: int)", replacement); Assert.Contains("function Transform(Value: string)", replacement);
        Assert.Contains("F: function(int): int := Map", replacement); Assert.Contains("F(Map(Value := 40)) + Transform(Label)", replacement);
        Assert.Contains("// Transform stays", replacement); Assert.Contains("'Transform'", replacement);
        await analysis.Context.VerifyInputsAsync();
    }

    [Fact]
    public async Task Compiling_successfully_does_not_allow_capture_of_an_unrelated_binding()
    {
        const string source = """
            unit Demo.Bridge;
            function First(): int; begin return 1; end function;
            function Second(): int; begin return 2; end function;
            public function Run(): int;
              function Nested(): int;
              begin const Alias: function(): int := Second; return Local() + Alias(); end function;
            begin
              const Local: function(): int := First;
              return Nested();
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source); var analysis = await RenameFixture.Analyze(directory);
        var failure = Assert.Throws<ContextFailure>(() => RenameService.Prepare(analysis, RenameFixture.Select(directory, source, "Local: function"), "Second"));
        Assert.Equal("rename_conflict", failure.Code); Assert.Equal(source, File.ReadAllText(directory.File("Game/Bridge.fpas")));
    }

    [Theory]
    [InlineData("X: T", "X", "end")]
    [InlineData("Point<T> =", "Point", "record")]
    [InlineData("Value: int", "Value", "Input")]
    public async Task Generic_records_properties_constructor_labels_and_parameters_keep_bindings(string marker, string oldName, string newName)
    {
        const string source = """
            unit Demo.Bridge;
            public type Point<T> = record public X: T; end record;
            public function Read(Value: int): int;
            begin
              const P: Point<int> := new Point<int>(X := Value);
              return P.X;
            end function;
            public function Run(): int; begin return Read(Value := 42); end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source); var analysis = await RenameFixture.Analyze(directory);
        var prepared = RenameService.Prepare(analysis, RenameFixture.Select(directory, source, marker), newName);
        Assert.True(prepared.Plan.EditCount >= 3); Assert.True(prepared.Plan.ChangesPublicApi);
        var spelling = newName is "end" or "record" ? "@" + newName : newName;
        Assert.All(prepared.Plan.Files.SelectMany(file => file.Edits), edit => { Assert.Equal(oldName, edit.ExpectedText); Assert.Equal(spelling, edit.NewText); });
        await SourcePublication.ApplyAsync(prepared.Edits.ToSourceChanges(), analysis.Context.VerifyInputsAsync);
        Assert.True((await RenameFixture.Analyze(directory)).Success);
    }

    [Fact]
    public async Task Generic_constraint_parameter_references_and_noop_renames_are_preserved()
    {
        const string source = "unit Demo.Bridge; public type Item<T> = where T: class; record public Value: T; end record; end unit;";
        using var directory = RenameFixture.Create(sdk, source); var analysis = await RenameFixture.Analyze(directory);
        var selection = RenameFixture.Select(directory, source, "T> =");
        var changed = RenameService.Prepare(analysis, selection, "U"); Assert.Equal(3, changed.Plan.EditCount);
        var noop = RenameService.Prepare(analysis, selection, "T"); Assert.Empty(noop.Edits.Documents); Assert.Equal(0, noop.Plan.EditCount);
        Assert.Equal(source, File.ReadAllText(directory.File("Game/Bridge.fpas")));
    }

    [Fact]
    public async Task Source_interface_members_and_implementations_form_one_family_and_CLR_contracts_are_refused()
    {
        const string source = """
            unit Demo.Bridge;
            public type IReader = interface function Read(): int; end interface;
            public type Reader = record implements IReader
              public Value: int;
              public function Read(): int; begin return self.Value; end function;
            end record;
            public function Run(): int;
            begin const R: Reader := new Reader(Value := 21); const I: IReader := R; return R.Read() + I.Read(); end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source); var analysis = await RenameFixture.Analyze(directory);
        var prepared = RenameService.Prepare(analysis, RenameFixture.Select(directory, source, "Read(): int; end interface"), "Fetch");
        Assert.Equal(2, prepared.Plan.Targets.Count); Assert.Equal(4, prepared.Plan.EditCount);
        const string external = "unit Demo.Bridge; uses System as S; public type Resource = record implements S.IDisposable public procedure Dispose(); begin end procedure; end record; end unit;";
        File.WriteAllText(directory.File("Game/Bridge.fpas"), external); analysis = await RenameFixture.Analyze(directory);
        Assert.Equal("rename_external_contract", Assert.Throws<ContextFailure>(() => RenameService.Prepare(analysis,
            RenameFixture.Select(directory, external, "Dispose();"), "Close")).Code);
    }
}
