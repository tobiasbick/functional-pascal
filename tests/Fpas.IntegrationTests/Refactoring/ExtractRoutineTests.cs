// [Documentation](../../../docs/pascal/tools/refactoring.md#extract-a-routine)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;

namespace Fpas.IntegrationTests.Refactoring;

/// <summary>Verifies checked extraction of a sequence into a reusable helper.</summary>
/// <param name="sdk">The matching SDK package for test projects.</param>
public sealed class ExtractRoutineTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Applies a checked extraction with a live write and an escaping final local.</summary>
    [Fact]
    public async Task Cli_extracts_a_result_and_preserves_capture_mutation_and_order_at_runtime()
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

            function Compute(Start: int): int;
            begin
              var Total: int := Start;
              Total := Total + Mark(1);
              const Result: int := Total + Mark(2);
              return Result + Total + Mark(3);
            end function;

            begin
              Console.WriteLine(Compute(5));
              Console.WriteLine(Trace);
            end;
            end program;
            """.ReplaceLineEndings("\n");
        using var directory = RenameFixture.Create(sdk, source);
        const string project = "Game/Game.fpasproj";
        File.WriteAllText(directory.File(project), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>
            """);
        var at = RenameFixture.Select(directory, source, "Total := Total");
        var stop = RenameFixture.Select(directory, source, ";\n  return Result");
        var path = directory.File(".temp-data/routine.json");
        var preview = await RenameFixture.Cli(directory, "extract-routine", project, "--file", at.File,
            "--line", Number(at.Line), "--column", Number(at.Column), "--end-line", Number(stop.Line),
            "--end-column", Number(stop.Column + 1), "--name", "Measure", "--plan", path, "--report", "json");
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("extract-routine", plan.Kind);
        Assert.Equal(new RefactorPosition(stop.Line, stop.Column + 1), plan.SelectionEnd);
        Assert.Contains(plan.Notes!, note => note.Contains("writes to outer storage for Total", StringComparison.Ordinal));
        Assert.Contains(plan.Notes!, note => note.Contains("Return the initializer for Result", StringComparison.Ordinal));
        Assert.Equal(source, File.ReadAllText(at.File));

        var applied = await RenameFixture.Cli(directory, "apply", project, "--plan", path, "--no-restore");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        var edited = File.ReadAllText(at.File);
        Assert.Contains("  function Measure(): int;\n  begin\n    Total := Total + Mark(1);\n    return Total + Mark(2);\n  end function;\nbegin", edited);
        Assert.Contains("  const Result: int := Measure();\n  return Result + Total + Mark(3);", edited);
        var built = await sdk.Run(directory, "build", project, "--no-restore", "-m:1", "--disable-build-servers");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var executed = await ProcessRunner.RunAsync([directory.File("Game/bin/Debug/net10.0/Game.dll")]);
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("17\n123", executed.Output.ReplaceLineEndings("\n").Trim());

        static string Number(int value) => FormattableString.Invariant($"{value}");
    }

    /// <summary>Checks nested block placement, captured name binding and preservation of strings and comments.</summary>
    [Fact]
    public async Task Keeps_block_control_flow_and_multiline_literal_text()
    {
        var source = """
            unit Demo.Bridge;
            uses System.Text as Text;
            function Compute(Flag: bool): string;
            begin
              var Answer: string := '';
              if Flag then
                const Builder: Text.StringBuilder := new Text.StringBuilder();
                while true do
                  // Record the result.
                  discard Builder.Append('first
             second');
                  break;
                end while;
                Answer := Builder.ToString();
              end if;
              return Answer;
            end function;
            end unit;
            """.ReplaceLineEndings("\r\n");
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var outcome = Prepare("while true", "end while;", "Fill");
        var result = Assert.Single(outcome.Edits.Documents).NewText;
        Assert.Contains("    procedure Fill();\r\n    begin\r\n      while true do\r\n        // Record the result.", result);
        Assert.Contains("'first\r\n second'", result);
        Assert.Contains("    end procedure;\r\n\r\n    Fill();\r\n    Answer", result);
        Assert.DoesNotContain('\n', result.Replace("\r\n", "", StringComparison.Ordinal));
        Assert.Contains(outcome.Notes, note => note.Contains("Capture outer bindings Builder", StringComparison.Ordinal));
        Assert.Contains("loop targeted", Assert.Throws<ContextFailure>(() => Prepare("break;", "break;", "Stop")).Message);
        Assert.Contains("complete", Assert.Throws<ContextFailure>(() => Prepare("true do", "end while;", "Bad")).Message);

        PreparedRefactoring Prepare(string first, string last, string name)
        {
            var start = RenameFixture.Select(directory, source, first);
            var end = RenameFixture.Select(directory, source, last);
            return CheckedRefactoring.Prepare(analysis, new("extract-routine", start, end.Line, end.Column + last.Length, Name: name));
        }
    }

    /// <summary>Checks tail returns and refusals when a local, resource, or return would cross its boundary.</summary>
    [Fact]
    public async Task Handles_complete_tail_returns_and_refuses_boundary_changes()
    {
        var source = """
            unit Demo.Bridge;
            uses System.IO as IO;
            function Choose(Value: int): int;
            begin
              if Value < 0 then
                return 0;
              end if;
              return Value;
            end function;

            function Escape(): int;
            begin
              const First: int := 1;
              const Second: int := 2;
              return First + Second;
            end function;

            function Managed(ref Number: int): int;
            begin
              Number := Number + 1;
              return Number;
            end function;

            function Resource(): int;
            begin
              using const Stream: IO.MemoryStream := new IO.MemoryStream();
              return 1;
            end function;
            end unit;
            """.ReplaceLineEndings("\n");
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var tail = Assert.Single(Prepare("if Value", "return Value;").Edits.Documents).NewText;
        Assert.Contains("  function Calculate(): int;", tail);
        Assert.Contains("    return Value;\n  end function;\nbegin\n  return Calculate();", tail);
        Assert.Contains("ends in a return", Assert.Throws<ContextFailure>(() => Prepare("if Value", "end if;")).Message);
        Assert.Contains("sole new result", Assert.Throws<ContextFailure>(() => Prepare("const First:", "const Second: int := 2;")).Message);
        Assert.Contains("cannot be captured", Assert.Throws<ContextFailure>(() => Prepare("Number := Number", "return Number;")).Message);
        Assert.Contains("complete block", Assert.Throws<ContextFailure>(() => Prepare("using const Stream", "new IO.MemoryStream();")).Message);
        Assert.Equal(source, File.ReadAllText(directory.File("Game/Bridge.fpas")));

        PreparedRefactoring Prepare(string first, string last)
        {
            var start = RenameFixture.Select(directory, source, first);
            var end = RenameFixture.Select(directory, source, last);
            return CheckedRefactoring.Prepare(analysis, new("extract-routine", start, end.Line, end.Column + last.Length, Name: "Calculate"));
        }
    }
}
