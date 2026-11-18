using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using System.Globalization;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class RemoveParameterTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Cli_previews_and_applies_one_checked_removal_across_named_positional_and_default_arguments()
    {
        const string source = """
            program TraceCalls;
            uses System.Console as Console;
            var Trace: int := 0;
            function Step(N: int): int; begin Trace := Trace * 10 + N; return N; end function;
            function Choose(Keep: int; Unused: int := 0; Tail: int := 1): int;
            begin return Keep + Tail; end function;
            function Run(): int;
            begin
              const Cached: int := 4;
              return Choose(Step(1), 9, Step(3)) + Choose(Tail := Step(8), Unused := Cached, Keep := Step(4)) + Choose(Step(5));
            end function;
            begin Console.WriteLine(Run()); Console.WriteLine(Trace); end;
            end program;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var project = directory.File("Game/Game.fpasproj");
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>");
        var baseline = await Execute();
        Assert.Equal(new[] { "22", "13845" }, baseline);
        var at = RenameFixture.Select(directory, source, "Unused := Cached");
        var path = directory.File(".temp-data/remove.json");
        var preview = await RenameFixture.Cli(directory, "remove-parameter", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas",
            "--line", at.Line.ToString(CultureInfo.InvariantCulture), "--column", at.Column.ToString(CultureInfo.InvariantCulture),
            "--plan", path, "--report", "json");
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        Assert.Equal(source, File.ReadAllText(at.File));
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("remove-parameter", plan.Kind);
        Assert.Equal("Unused", Assert.Single(plan.Targets).Name);
        Assert.Equal(3, plan.EditCount);
        Assert.False(plan.ChangesPublicApi);
        var result = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore", "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var expected = source.Replace("Unused: int := 0; ", "").Replace("9, ", "").Replace("Unused := Cached, ", "");
        Assert.Equal(expected, File.ReadAllText(at.File));
        var stale = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore", "--report", "json");
        Assert.Equal(1, stale.ExitCode);
        Assert.Contains("stale_plan", stale.Output);
        Assert.Equal(expected, File.ReadAllText(at.File));
        Assert.Equal(baseline, await Execute());

        async Task<string[]> Execute()
        {
            var build = await sdk.Run(directory, "build", project, "-m:1", "--disable-build-servers");
            Assert.True(build.ExitCode == 0, build.Output + build.Error);
            var run = await ProcessRunner.RunAsync([directory.File("Game/bin/Debug/net10.0/Game.dll")]);
            Assert.True(run.ExitCode == 0, run.Output + run.Error);
            return run.Output.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries);
        }
    }

    [Fact]
    public async Task Refuses_used_parameters_indirect_calls_effects_possible_exceptions_and_collisions()
    {
        const string source = """
            unit Demo.Bridge;
            function ValueOf(): int; begin return 1; end function;
            function ReadValue(Used: int): int; begin return Used; end function;
            function Contract(Precondition: int): int; requires Precondition > 0; begin return 1; end function;
            function Captures(Captured: int): int;
              function Inner(): int; begin return Captured; end function;
            begin return Inner(); end function;
            procedure ByReference(ref Storage: int); begin end procedure;
            public function Exposed(PublicSlot: int): int; begin return 1; end function;
            function Indirect(DelegateSlot: int): int; begin return 1; end function;
            function Effect(Invocation: int): int; begin return 2; end function;
            function Narrow(Conversion: byte): int; begin return 1; end function;
            function Divide(Arithmetic: int): int; begin return 3; end function;
            function Collision(Overload: int): int; begin return 4; end function;
            function Collision(): int; begin return 5; end function;
            function Generic<T>(Inferred: T): int; begin return 6; end function;
            function Annotated(Note: int; // Document this parameter.
              Keep: int): int; begin return Keep; end function;
            public function Run(): int;
            begin
              const Callback: function(int): int := Indirect;
              const N: int := ValueOf();
              return Callback(N) + ReadValue(N) + Effect(ValueOf()) + Divide(42 div N) +
                Collision(0) + Collision() + Generic(2) + Annotated(1, 2) + Narrow(cast<byte>(N));
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        foreach (var (marker, code) in new[] {
            ("Used: int", "refactor_not_supported"), ("DelegateSlot:", "refactor_not_supported"),
            ("Precondition:", "refactor_not_supported"), ("Captured:", "refactor_not_supported"),
            ("Storage:", "refactor_not_supported"), ("PublicSlot:", "refactor_not_supported"),
            ("Invocation:", "refactor_not_supported"), ("Arithmetic:", "refactor_not_supported"),
            ("Conversion:", "refactor_not_supported"), ("Note:", "refactor_not_supported"), ("Overload:", "refactor_conflict"), ("Inferred:", "refactor_conflict") })
        {
            var at = RenameFixture.Select(directory, source, marker);
            var error = Assert.Throws<ContextFailure>(() => CheckedRefactoring.Prepare(analysis, new("remove-parameter", at, at.Line, at.Column)));
            Assert.True(error.Code == code, marker + " " + error.Code + " " + error.Message);
        }
        Assert.Equal(source, File.ReadAllText(directory.File("Game/Bridge.fpas")));
    }
}
