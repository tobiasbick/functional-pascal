// [Documentation](../../../docs/pascal/tools/refactoring.md#inline-a-local-variable)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;

namespace Fpas.IntegrationTests.Refactoring;

/// <summary>Checks that local inlining preserves types, binding and evaluation.</summary>
/// <param name="sdk">The matching SDK package for the test projects.</param>
public sealed class InlineVariableTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Applies an effectful initializer at its first use and verifies the call trace at runtime.</summary>
    [Fact]
    public async Task Cli_inlines_one_effectful_read_in_the_next_statement()
    {
        var source = """
            program Demo.Main;
            uses System.Console as Console;
            var Trace: int := 0;

            function Mark(Digit: int): int;
            begin
              Trace := Trace * 10 + Digit;
              return Digit;
            end function;

            function Pair(First: int; Second: int): int;
            begin
              return First * 10 + Second;
            end function;

            begin
              const Left: int := Mark(1);
              Console.WriteLine(Pair(Left, Mark(2)));
              Console.WriteLine(Trace);
            end;

            end program;
            """.ReplaceLineEndings("\n");
        using var directory = RenameFixture.Create(sdk, source);
        const string project = "Game/Game.fpasproj";
        File.WriteAllText(directory.File(project), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>
            """);
        var at = RenameFixture.Select(directory, source, "Left, Mark");
        var path = directory.File(".temp-data/inline.json");
        var preview = await RenameFixture.Cli(directory, "inline-variable", project, "--file", at.File,
            "--line", FormattableString.Invariant($"{at.Line}"), "--column", FormattableString.Invariant($"{at.Column}"),
            "--plan", path, "--report", "json");
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("inline-variable", plan.Kind);
        Assert.Equal("Left", Assert.Single(plan.Targets).Name);
        Assert.Equal(source, File.ReadAllText(at.File));

        var applied = await RenameFixture.Cli(directory, "apply", project, "--plan", path, "--no-restore");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        Assert.Equal(source.Replace("  const Left: int := Mark(1);\n", "").Replace("Pair(Left,", "Pair((Mark(1)),"),
            File.ReadAllText(at.File));
        var built = await sdk.Run(directory, "build", project, "--no-restore", "-m:1", "--disable-build-servers");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var result = await ProcessRunner.RunAsync([directory.File("Game/bin/Debug/net10.0/Game.dll")]);
        Assert.Equal(0, result.ExitCode);
        Assert.Equal("12\n12", result.Output.ReplaceLineEndings("\n").Trim());
    }

    /// <summary>Inlines multiple reads of a literal and separately preserves comments and an unchecked expression.</summary>
    [Fact]
    public async Task Keeps_exact_bindings_grouping_and_line_endings()
    {
        var source = """
            unit Demo.Bridge;

            function Example(): int;
            begin
              const Base: int := 41;
              if Base > 0 then
                const Base: int := 9;
                return Base;
              end if;
              return Base + 1;
            end function;

            function Group(Value: int): int;
            begin
              const Total: int := unchecked(Value + 1);
              // Keep this comment.
              return Total * 2;
            end function;

            end unit;
            """.ReplaceLineEndings("\r\n");
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var literal = Prepare("Base: int := 41");
        var result = Assert.Single(literal.Edits.Documents).NewText;
        Assert.Contains("  if (41) > 0 then\r\n    const Base: int := 9;\r\n    return Base;", result);
        Assert.Contains("return (41) + 1;", result);
        Assert.DoesNotContain("41;", result);
        var group = Assert.Single(Prepare("Total:").Edits.Documents).NewText;
        Assert.Contains("  // Keep this comment.\r\n  return (unchecked(Value + 1)) * 2;", group);
        Assert.DoesNotContain('\n', group.Replace("\r\n", "", StringComparison.Ordinal));

        PreparedRefactoring Prepare(string marker)
        {
            var at = RenameFixture.Select(directory, source, marker);
            return CheckedRefactoring.Prepare(analysis, new("inline-variable", at, at.Line, at.Column));
        }
    }

    /// <summary>Refuses changes to timing, writeable storage, conversions, and comments.</summary>
    [Fact]
    public async Task Refuses_reads_that_need_storage_or_move_effects()
    {
        const string source = """
            unit Demo.Bridge;
            uses System as Sys;
            function Next(): int;
            begin
              return 7;
            end function;
            function Pair(First: int; Second: int): int;
            begin
              return First + Second;
            end function;
            function InCheck(in Value: int): int;
            begin
              return Value;
            end function;
            function Later(): int;
            begin
              const Delayed: int := Next();
              discard Next();
              return Delayed;
            end function;
            function AfterEffect(): int;
            begin
              const AfterFirst: int := Next();
              return Pair(Next(), AfterFirst);
            end function;
            function Conditional(Condition: bool): int;
            begin
              const Maybe: int := Next();
              if Condition then
                return Maybe;
              end if;
              return 0;
            end function;
            function Reused(): int;
            begin
              const Twice: int := Next();
              return Twice + Twice;
            end function;
            function Edited(): int;
            begin
              var Changed: int := 7;
              Changed := 8;
              return Changed;
            end function;
            function Address(): int;
            begin
              const Stored: int := 9;
              return InCheck(in Stored);
            end function;
            function Converted(): long;
            begin
              const Wide: long := 7;
              return Wide;
            end function;
            function Documented(): int;
            begin
              const Attached: int := 9; // Keep this note.
              return Attached;
            end function;
            function Receiver(ReceiverValue: Sys.Random): int;
            begin
              const AfterReceiver: int := Next();
              return ReceiverValue.Next(AfterReceiver);
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        foreach (var marker in new[] { "Delayed:", "AfterFirst:", "Maybe:", "Twice:", "AfterReceiver:" })
            Assert.Contains("evaluated first", Failure(marker).Message);
        Assert.Contains("writes", Failure("Changed:").Message);
        Assert.Equal("refactor_conflict", Failure("Stored:").Code);
        Assert.Contains("storage conversion", Failure("Wide:").Message);
        Assert.Contains("comment", Failure("Attached:").Message);
        Assert.Equal(source, File.ReadAllText(directory.File("Game/Bridge.fpas")));

        ContextFailure Failure(string marker)
        {
            var at = RenameFixture.Select(directory, source, marker);
            return Assert.Throws<ContextFailure>(() => CheckedRefactoring.Prepare(analysis, new("inline-variable", at, at.Line, at.Column)));
        }
    }
}
