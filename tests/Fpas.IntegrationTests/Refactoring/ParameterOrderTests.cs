// [Documentation](../../../docs/pascal/tools/refactoring.md#reorder-parameters)
using System.Text.Json;
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;

namespace Fpas.IntegrationTests.Refactoring;

/// <summary>Exercises checked parameter permutations and execution after applying a plan.</summary>
/// <param name="sdk">The matching SDK package for the test projects.</param>
public sealed class ParameterOrderTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Checks cross-project labeling, unchanged defaults and the side-effect trace at runtime.</summary>
    [Fact]
    public async Task Cli_reorders_public_parameters_without_reordering_side_effects()
    {
        const string library = """
            unit Demo.Bridge;
            public function Encode(Left: int; Right: int; Weight: int := 9; Offset: int := 7): int;
            begin
              return Left * 1000 + Right * 100 + Weight * 10 + Offset;
            end function;
            public function Encode(Text: string; Value: int): int;
            begin
              return Value;
            end function;
            end unit;
            """;
        const string caller = """
            program Consumer;
            uses Demo.Bridge as B; uses System.Console as Console;
            var Trace: int := 0;

            function Mark(Digit: int): int;
            begin
              Trace := Trace * 10 + Digit;
              return Digit;
            end function;

            begin
              Console.WriteLine(B.Encode(Mark(1), Mark(2), Offset := Mark(3)));
              Console.WriteLine(B.Encode(Right := Mark(4), Left := Mark(5)));
              Console.WriteLine(Trace);
              Console.WriteLine(B.Encode('other', 6));
            end;
            end program;
            """;
        using var directory = RenameFixture.Create(sdk, library);
        const string project = "Consumer/Consumer.fpasproj";
        const string callerFile = "Consumer/Main.fpas";
        Directory.CreateDirectory(directory.File("Consumer"));
        File.WriteAllText(directory.File(project), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Exe</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File(callerFile), caller);
        var at = RenameFixture.Select(directory, caller, "Encode(Mark", callerFile);
        var planPath = directory.File(".temp-data/order.json");
        var options = new[] { "change-signature", project, "--file", at.File,
            "--line", FormattableString.Invariant($"{at.Line}"), "--column", FormattableString.Invariant($"{at.Column}"),
            "--order", "Right,Left,Offset,Weight", "--plan", planPath, "--report", "json" };

        var incompatible = await RenameFixture.Cli(directory, [.. options, "--mode", "ref"]);
        Assert.Equal(2, incompatible.ExitCode);
        Assert.False(File.Exists(planPath));
        var preview = await RenameFixture.Cli(directory, [.. options, "--closed-context"]);
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(planPath));
        Assert.Equal(new[] { "Right", "Left", "Offset", "Weight" }, plan.SignatureChange!.Order);
        Assert.True(plan.ChangesPublicApi);
        Assert.True(plan.ClosedContext);
        Assert.Equal(2, plan.Files.Count);
        Assert.Equal(caller, File.ReadAllText(at.File));

        var applied = await RenameFixture.Cli(directory, "apply", project, "--plan", planPath, "--no-restore", "--report", "json");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        using (var result = JsonDocument.Parse(applied.Output))
            Assert.True(result.RootElement.GetProperty("rechecked").GetBoolean());
        Assert.Equal(caller.Replace("Encode(Mark(1), Mark(2)", "Encode(Left := Mark(1), Right := Mark(2)"),
            File.ReadAllText(at.File));
        Assert.Equal(library.Replace("Left: int; Right: int; Weight: int := 9; Offset: int := 7",
            "Right: int; Left: int; Offset: int := 7; Weight: int := 9"),
            File.ReadAllText(directory.File("Game/Bridge.fpas")));

        var built = await sdk.Run(directory, "build", project, "--no-restore", "-m:1", "--disable-build-servers");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var executed = await ProcessRunner.RunAsync([directory.File("Consumer/bin/Debug/net10.0/Consumer.dll")]);
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("1293\n5497\n12345\n6", executed.Output.ReplaceLineEndings("\n").Trim());
    }

    /// <summary>Preserves argument modes, generic substitutions, stable positional prefixes and nested call order.</summary>
    [Fact]
    public async Task Keeps_nested_calls_in_place_and_refuses_ambiguous_parameter_comments()
    {
        const string source = """
            unit Demo.Bridge;
            function Use of T(Scale: int; ref State: int; Item: T; Count: int := 3): T;
            begin
              State := State + Scale * Count;
              return Item;
            end function;
            function Commented(First: int; // First or Second
              Second: int): int;
            begin
              return First + Second;
            end function;
            function CommentedOpening( // First or the routine
              First: int; Second: int): int;
            begin
              return First + Second;
            end function;
            function Overload(First: int; Second: string): int;
            begin
              return First;
            end function;
            function Overload(First: string; Second: int): int;
            begin
              return Second;
            end function;
            public function Run(): string;
            begin
              var Counter: int := 0;
              return Use(2, ref Counter, Use(1, ref Counter, 'answer', Count := 5));
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var prepared = Prepare("Use of T", "Scale", "Item", "State", "Count");
        var document = Assert.Single(prepared.Edits.Documents);
        Assert.Contains("Use of T(Scale: int; Item: T; ref State: int; Count: int := 3)", document.NewText);
        Assert.Contains("Use(2, State := ref Counter, Item := Use(1, State := ref Counter, Item := 'answer', Count := 5))",
            document.NewText);
        Assert.Equal(0, Prepare("Use of T", "Scale", "State", "Item", "Count").Edits.EditCount);
        Assert.Contains("exactly once", Assert.Throws<ContextFailure>(() => Prepare("Use of T", "Scale", "Unknown", "State", "Count")).Message);
        Assert.Contains("required parameters", Assert.Throws<ContextFailure>(() => Prepare("Use of T", "Count", "Scale", "State", "Item")).Message);
        Assert.Contains("comment", Assert.Throws<ContextFailure>(() => Prepare("Commented(", "Second", "First")).Message);
        Assert.Contains("comment", Assert.Throws<ContextFailure>(() => Prepare("CommentedOpening(", "Second", "First")).Message);
        Assert.Equal("refactor_conflict", Assert.Throws<ContextFailure>(() => Prepare("Overload(", "Second", "First")).Code);

        PreparedRefactoring Prepare(string marker, params string[] order)
        {
            var at = RenameFixture.Select(directory, source, marker);
            return CheckedRefactoring.Prepare(analysis, new("change-signature", at, at.Line, at.Column,
                SignatureChange: new(null, null, [], order)));
        }
    }
}
