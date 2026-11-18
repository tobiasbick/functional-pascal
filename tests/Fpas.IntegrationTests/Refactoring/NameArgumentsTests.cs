using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Globalization;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class NameArgumentsTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Naming_a_mixed_call_preserves_nested_effects_comments_and_written_order()
    {
        const string source = """
            program Order;
            uses System.Console as Console;
            var Trace: int := 0;
            function Step(N: int): int; begin Trace := Trace * 10 + N; return N; end function;
            function Mix(@end: int; B: int; C: int): int; begin return @end * 100 + B * 10 + C; end function;
            begin
              Console.WriteLine(Mix( // keep the order
                Step(1), C := Step(2), B := Step(3)));
              Console.WriteLine(Trace);
            end;
            end program;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var project = directory.File("Game/Game.fpasproj");
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>");
        var before = await Execute();
        Assert.Equal(new[] { "132", "123" }, before);
        var selection = RenameFixture.Select(directory, source, "Mix( //");
        var path = directory.File(".temp-data/naming.json");
        var preview = await RenameFixture.Cli(directory, "name-arguments", project, "--file", selection.File,
            "--line", selection.Line.ToString(CultureInfo.InvariantCulture), "--column", selection.Column.ToString(CultureInfo.InvariantCulture),
            "--plan", path, "--report", "json");
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("name-arguments", plan.Kind);
        Assert.Equal("@end := ", Assert.Single(Assert.Single(plan.Files).Edits).NewText);
        Assert.Equal(source, File.ReadAllText(selection.File));
        var applied = await RenameFixture.Cli(directory, "apply", project, "--plan", path, "--no-restore");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        Assert.Equal(source.Replace("Step(1), C :=", "@end := Step(1), C :="), File.ReadAllText(selection.File));
        var request = new RefactoringRequest("name-arguments", selection, selection.Line, selection.Column);
        Assert.Equal(before, await Execute());
        var changed = await RenameFixture.Analyze(directory);
        Assert.Equal("refactor_not_supported", Assert.Throws<ContextFailure>(() => CheckedRefactoring.Prepare(changed, request)).Code);

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
    public async Task Uses_concrete_slots_for_CLR_calls_constructors_delegates_byrefs_and_params()
    {
        const string source = """
            unit Demo.Bridge;
            uses Sample as S;
            function Pair<T>(Left: T; Right: T): T; begin return Left; end function;
            public function Run(Choose: S.Choose; Text: string): int;
            begin
              var N: int := 1;
              const Items: array of int := [1, 2];
              const Bytes: array of byte := [1, 2];
              S.Api.Copy(ref N, out N);
              S.Api.Optional(1);
              S.Api.Variadic(1, 2, 3);
              S.Api.Variadic(1, 2, Items);
              S.Api.Variadic(2, 3, [4]);
              S.Api.Variadic(4, 5);
              S.Api.Variadic(1, 2, 3, 4);
              S.Api.Route(1, 2, Text);
              S.Api.Send(Bytes, Bytes, 1);
              const A: S.Api := new S.Api(true, false);
              return Choose(N, Pair<int>(2, 3)) + Text.Substring(1, 2).Length;
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        File.WriteAllText(directory.File("Shared/CallSites.cs"), """
            namespace Sample;
            public delegate int Choose(int first, int second);
            public class Api
            {
                public Api(bool enabled, bool visible) { }
                public static void Copy(ref int source, out int result) => result = source;
                public static void Optional(int x, int y = 0) { }
                public static void Variadic(int x, int y, params int[] rest) { }
                public static void Route(int x, int y, string text) { }
                public static void Route(string text, int x, int y) { }
                public static void Send(System.ReadOnlySpan<byte> block, byte[] tail, int count) { }
                public static void Send(int count, byte[] tail, byte[] block) { }
            }
            """);
        var analysis = await RenameFixture.Analyze(directory);
        PreparedRefactoring Prepare(string marker)
        {
            var at = RenameFixture.Select(directory, source, marker);
            return CheckedRefactoring.Prepare(analysis, new("name-arguments", at, at.Line, at.Column));
        }
        void Check(string marker, string replacement) =>
            Assert.Equal(source.Replace(marker, replacement), Assert.Single(Prepare(marker).Edits.Documents).NewText);

        Check("Pair<int>(2, 3)", "Pair<int>(Left := 2, Right := 3)");
        Check("Choose(N, Pair<int>(2, 3))", "Choose(first := N, second := Pair<int>(2, 3))");
        Check("Substring(1, 2)", "Substring(startIndex := 1, length := 2)");
        Check("new S.Api(true, false)", "new S.Api(enabled := true, visible := false)");
        Check("Copy(ref N, out N)", "Copy(source := ref N, result := out N)");
        Check("Variadic(1, 2, Items)", "Variadic(x := 1, y := 2, rest := Items)");
        Check("Variadic(2, 3, [4])", "Variadic(x := 2, y := 3, rest := [4])");
        Check("Variadic(4, 5)", "Variadic(x := 4, y := 5)");
        foreach (var marker in new[] { "Optional(1)", "Variadic(1, 2, 3)", "Variadic(1, 2, 3, 4)" })
            Assert.Equal("refactor_not_supported", Assert.Throws<ContextFailure>(() => Prepare(marker)).Code);
        foreach (var marker in new[] { "Route(1, 2, Text)", "Send(Bytes, Bytes, 1)" })
            Assert.Equal("refactor_conflict", Assert.Throws<ContextFailure>(() => Prepare(marker)).Code);
    }
}
