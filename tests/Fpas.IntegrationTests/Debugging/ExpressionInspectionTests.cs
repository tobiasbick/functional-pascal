using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class ExpressionInspectionTests
{
    [Fact]
    public async Task Passive_expressions_read_one_stop_follow_fpas_scalar_rules_and_isolate_errors()
    {
        using var program = new Debuggee("own-passive-expressions");
        var source = """
            program Expressions;
            uses System.Console as Console, System.IO.File as File, System.String as Strings;
            var Global: int := 21;
            type Point = record public X: int; end record;
            type UserId = distinct long;
            procedure Observe(Input: int);
            begin
              const Global: int := 99;
              const Value: int := 41;
              const @array: int := 7;
              const Unsigned: uint := 42U;
              const NativeSigned: nint := cast<nint>(41);
              const NativeUnsigned: nuint := cast<nuint>(41);
              const Amount: decimal := 1.2500M;
              const Wide: long := 16777217L;
              const Rounded: float := 16777216.0F;
              const Text: string := 'λ😀';
              const LongText: string := new Strings(#955, 5000);
              const Position: Point := new Point(42);
              const Points: array of Point := [new Point(42)];
              const Id: UserId := new UserId(41L);
              const Values: array of int := [10, 20, 30];
              const Grid: array[,] of int := new int[2, 3];
              Grid[1, 2] := 42;
              Console.WriteLine('observe');
              File.WriteAllText('after.txt', 'executed');
            end procedure;
            begin Observe(7); end;
            end program;
            """;
        await program.Compile(source);
        var line = Array.FindIndex(source.Split('\n'), line => line.Contains("Console.WriteLine('observe')", StringComparison.Ordinal)) + 1;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(40));
        await using var host = await DebugSessionHost.Start(WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost()), program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, line)), deadline.Token);
        await host.Initialize(deadline.Token); var stop = await host.WaitForStop(deadline.Token); Assert.Equal("breakpoint", stop.Reason);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var frame = (await host.Stack(new(stop.StopId.Value, thread.Id), deadline.Token)).Items[0];
        foreach (var (expression, type, expected) in new[]
        {
            ("Value + 1", "int", "42"), ("Input * (2 + 4)", "int", "42"), ("Global", "int", "99"), ("Expressions.Global", "int", "21"),
            ("1_000 div 3", "int", "333"), ("$FF and 15", "int", "15"), ("not 0", "int", "-1"), ("1 shl 33", "int", "2"),
            ("Unsigned + 1", "uint", "43"), ("NativeSigned + 1", "nint", "42"), ("NativeUnsigned + 1", "nuint", "42"),
            ("Amount + 1.25M", "decimal", "2.5000"), ("Wide - Rounded", "float", "0"), ("1.0 / 0.0", "double", "Infinity"),
            ("1 / 2", "double", "0.5"), ("NativeUnsigned / -2", "double", "-20.5"),
            ("false and (1 / 0 > 0.0)", "bool", "false"),
            ("-2147483648", "int", "-2147483648"), ("-9223372036854775808L", "long", "-9223372036854775808"),
            ("false and (1 div 0 = 0)", "bool", "false"), ("true or (1 div 0 = 0)", "bool", "true"), ("true xor false", "bool", "true"),
            ("(false and false) or true", "bool", "true"), ("false and (false or true)", "bool", "false"),
            ("false and (Points[1 div 0].X = 0)", "bool", "false"),
            ("'it''s' + #10", "string", "it's\n"), ("'first\r\nsecond\rlast''line'", "string", "first\nsecond\nlast'line"),
            ("@array", "int", "7"), ("Text[0]", "char", "λ"), ("Position.X * 2", "int", "84"), ("Id.Value + 1L", "long", "42"),
            ("Values[1 + 1]", "int", "30"), ("Grid[1, 2]", "int", "42"), ("int.MaxValue", "int", "2147483647"),
            ("string.Empty + 'x'", "string", "x"), ("null = null", "bool", "true"), ("LongText[4095]", "char", "λ")
        })
        {
            var result = await host.Evaluate(new(stop.StopId.Value, frame.Id, expression), deadline.Token);
            Assert.True(result.Error is null, $"{expression}: {result.Error}"); Assert.NotNull(result.Value);
            Assert.Equal(type, result.Value.Type); Assert.Equal(expected, result.Value.Scalar);
        }
        var surrogate = await host.Evaluate(new(stop.StopId.Value, frame.Id, "Text[1]"), deadline.Token);
        Assert.Null(surrogate.Error); Assert.Equal("D83D", surrogate.Value!.RawUtf16);
        foreach (var (expression, code) in new[]
        {
            ("Value div 0", "division_by_zero"), ("int.MaxValue + 1", "overflow"),
            ("Amount + 1.0", "unsupported_expression"), ("false and 1", "unsupported_expression"), ("Values[3]", "index_out_of_range"),
            ("false and (false) or true", "invalid_expression"),
            ("Text[-1]", "index_out_of_range"), ("value", "unresolved_name"), ("Position + Position", "unsupported_expression"),
            ("array", "unsupported_expression"),
            ("File.WriteAllText('bad.txt', 'bad')", "unsupported_expression"), ("new Point(1)", "unsupported_expression"),
            ("Value := 0", "unsupported_expression"), ("1.0L", "invalid_expression"),
            ("'abc' ?? 1", "unsupported_expression"), ("'abc' ?? MissingName", "unsupported_expression"), ("null ?? 1", "unsupported_expression"),
            ("false and (Points['bad'].X = 0)", "unsupported_expression"), ("false and (Points[MissingName].X = 0)", "unresolved_name"),
            ("false and (Points[0, 0].X = 0)", "invalid_expression"), ("LongText[4096]", "truncated_value"), ("LongText + ''", "truncated_value")
        })
        {
            var result = await host.Evaluate(new(stop.StopId.Value, frame.Id, expression), deadline.Token);
            Assert.Null(result.Value); Assert.Equal(code, result.Error?.Code); Assert.Equal(stop.Revision, host.Snapshot.Revision);
        }
        Assert.False(File.Exists(program.FilePath("after.txt"))); Assert.False(File.Exists(program.FilePath("bad.txt")));
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task Expression_limits_are_explicit_and_frame_references_expire_on_resume()
    {
        using var program = new Debuggee("own-expression-limits");
        await program.Compile("""
            program ExpressionLimits;
            uses System.Console as Console;
            procedure Observe(Value: int);
            begin
              Console.WriteLine(Value);
            end procedure;
            begin Observe(1); Observe(2); end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(40));
        await using var host = await DebugSessionHost.Start(WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost()), program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 5)), deadline.Token);
        await host.Initialize(deadline.Token); var stop = await host.WaitForStop(deadline.Token);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var frame = (await host.Stack(new(stop.StopId.Value, thread.Id), deadline.Token)).Items[0];
        foreach (var (expression, expected) in new[] { (Sum(1900), "1900"), (new string('(', 63) + "1" + new string(')', 63), "1"), ("@Value", "1"), ("1 // comment\n + 2", "3") })
        {
            var result = await host.Evaluate(new(stop.StopId.Value, frame.Id, expression), deadline.Token);
            Assert.Null(result.Error); Assert.Equal(expected, result.Value?.Scalar);
        }
        foreach (var expression in new[] { Sum(2048), new string('(', 64) + "1" + new string(')', 64), string.Concat(Enumerable.Repeat("not ", 65)) + "true" })
        {
            Assert.True(expression.Length <= 8192);
            var limited = await host.Evaluate(new(stop.StopId.Value, frame.Id, expression), deadline.Token);
            Assert.Equal("limit_exceeded", limited.Error?.Code); Assert.Equal(stop.Revision, host.Snapshot.Revision);
        }
        foreach (var expression in new[] { "", new string('x', 8193) })
            Assert.Equal("invalid_expression", (await host.Evaluate(new(stop.StopId.Value, frame.Id, expression), deadline.Token)).Error?.Code);
        Assert.Equal("unresolved_name", (await host.Evaluate(new(stop.StopId.Value, frame.Id, "@true"), deadline.Token)).Error?.Code);
        await host.Resume(stop.StopId.Value, "continue", deadline.Token); var next = await host.WaitForStop(deadline.Token);
        var nextThread = Assert.Single(await host.Threads(next.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var nextFrame = (await host.Stack(new(next.StopId.Value, nextThread.Id), deadline.Token)).Items[0];
        var stale = await host.Evaluate(new(next.StopId.Value, frame.Id, "Value"), deadline.Token); Assert.Equal("stale_reference", stale.Error?.Code);
        var oldStop = await Assert.ThrowsAsync<DebugException>(() => host.Evaluate(new(stop.StopId.Value, nextFrame.Id, "Value"), deadline.Token));
        Assert.Equal("stale_reference", oldStop.Code);
        var fresh = await host.Evaluate(new(next.StopId.Value, nextFrame.Id, "Value + 40"), deadline.Token);
        Assert.Null(fresh.Error); Assert.Equal("42", fresh.Value?.Scalar);
        await host.Resume(next.StopId.Value, "continue", deadline.Token); Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode); await host.Stop(deadline.Token);

        static string Sum(int count) => count <= 32 ? string.Join('+', Enumerable.Repeat("1", count)) : "(" + Sum(count / 2) + ")+(" + Sum(count - count / 2) + ")";
    }
}
