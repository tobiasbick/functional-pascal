using Fpas.Debugging.Core;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class SourceControlTests
{
    [Fact]
    public async Task Statements_on_one_line_keep_distinct_source_columns_while_stepping()
    {
        using var program = new Debuggee("own-same-line-step");
        const string text = "program SameLine; uses System.Console as Console; begin var Value: int := 0; Value := Value + 1; Value := Value + 2; Console.WriteLine(Value); end; end program;";
        await program.Compile(text);
        var firstColumn = text.IndexOf("Value := Value + 1", StringComparison.Ordinal) + 1;
        var secondColumn = text.IndexOf("Value := Value + 2", StringComparison.Ordinal) + 1;
        var outputColumn = text.IndexOf("Console.WriteLine", StringComparison.Ordinal) + 1;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        var point = Assert.Single(await host.SetBreakpoints(new(program.Source, [new(1, firstColumn)], host.Snapshot.StopId), deadline.Token));
        Assert.Equal(firstColumn, point.Actual!.Column);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], stop.StopId), deadline.Token);
        var second = await Step(host, stop, "over", 1, deadline.Token);
        Assert.Equal(secondColumn, second.Frame.Source!.Column);
        Assert.Contains(second.Values, item => item.Name == "Value" && item.Scalar == "1");
        var output = await Step(host, second.Stop, "over", 1, deadline.Token);
        Assert.Equal(outputColumn, output.Frame.Source!.Column);
        Assert.Contains(output.Values, item => item.Name == "Value" && item.Scalar == "3");
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task Loop_backedges_step_to_the_repeated_condition_with_current_values()
    {
        using var program = new Debuggee("own-loop-step");
        await program.Compile("""
            program LoopSteps;
            uses System.Console as Console;
            begin
              var Value: int := 0;
              while Value < 3 do
                Value := Value + 1;
              end while;
              Console.WriteLine(Value);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [new(6)], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], stop.StopId), deadline.Token);
        for (var expected = 1; expected <= 3; expected++)
        {
            var condition = await Step(host, stop, "over", 5, deadline.Token);
            Assert.Contains(condition.Values, item => item.Name == "Value" && item.Scalar == expected.ToString(System.Globalization.CultureInfo.InvariantCulture));
            var body = await Step(host, condition.Stop, "over", expected == 3 ? 8 : 6, deadline.Token);
            stop = body.Stop;
        }
        await host.Resume(stop.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("3", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Fact]
    public async Task Step_into_skips_a_source_routine_marked_step_through()
    {
        using var program = new Debuggee("own-step-through");
        await program.Compile("""
            program StepThrough;
            uses System.Console as Console, System.Diagnostics.DebuggerStepThroughAttribute as StepThroughAttribute;
            [StepThroughAttribute]
            function Hidden(Value: int): int;
            begin
              const Result: int := Value + 2;
              return Result;
            end function;
            begin
              const Answer: int := Hidden(40);
              Console.WriteLine(Answer);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [new(10)], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var call = await host.WaitForStop(deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], call.StopId), deadline.Token);
        var next = await Step(host, call, "into", 11, deadline.Token);
        Assert.DoesNotContain("Hidden", next.Frame.Name, StringComparison.Ordinal);
        Assert.Contains(next.Values, item => item.Name == "Answer" && item.Scalar == "42");
        await host.Stop(deadline.Token);
    }

}
