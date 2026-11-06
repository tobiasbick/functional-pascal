using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class ScalarInspectionTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task A_source_binding_prefix_limit_rejects_only_its_breakpoint(bool alreadyJitted)
    {
        using var program = new Debuggee("own-binding-prefix-limit");
        var parameters = string.Join("; ", Enumerable.Range(0, 80).Select(index => $"A{index}: int"));
        var arguments = string.Join(", ", Enumerable.Range(0, 80));
        var source = $$"""
            program BindingPrefix;
            uses System.Console as Console, System.Runtime.CompilerServices as Runtime;
            [Runtime.MethodImplAttribute(Runtime.MethodImplOptions.NoInlining)]
            procedure Consume({{parameters}});
            begin
              Console.WriteLine(A79);
            end procedure;
            [Runtime.MethodImplAttribute(Runtime.MethodImplOptions.NoInlining)]
            procedure Observe();
            begin
              const Removed: int := 42;
              Consume({{arguments}});
            end procedure;
            begin
              Observe();
              Console.WriteLine('ready');
              Observe();
            end;
            end program;
            """;
        await program.Compile(source, "--optimize");
        program.UseFullyOptimizedJit();
        var line = Array.FindIndex(source.Split('\n'), text => text.TrimStart().StartsWith("Consume(", StringComparison.Ordinal)) + 1;
        var initial = alreadyJitted ? Array.FindIndex(source.Split('\n'), text => text.Contains("Console.WriteLine('ready')", StringComparison.Ordinal)) + 1 : line;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, initial)), deadline.Token);
        await host.Initialize(deadline.Token);
        var exited = await host.WaitForStop(deadline.Token);
        if (alreadyJitted)
        {
            Assert.Equal("breakpoint", exited.Reason);
            var point = Assert.Single(await host.SetBreakpoints(new(program.Source, [new(line)], exited.StopId), deadline.Token));
            Assert.Equal("rejected", point.State); Assert.Equal("limit_exceeded", point.Error!.Code);
            await host.Resume(exited.StopId!.Value, "continue", deadline.Token);
            exited = await host.WaitForStop(deadline.Token);
        }
        Assert.Equal("exited", exited.State); Assert.Null(exited.Error); Assert.Equal(0, exited.ExitCode);
        var rejected = Assert.Single(exited.Breakpoints!);
        Assert.Equal("rejected", rejected.State); Assert.Equal("limit_exceeded", rejected.Error!.Code);
        await host.Stop(deadline.Token);
        Assert.Contains("79", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Fact]
    public async Task A_source_point_removed_by_full_optimization_is_rejected_without_failing_the_session()
    {
        using var program = new Debuggee("own-removed-source-point");
        await program.Compile("""
            program RemovedPoint;
            uses System.Console as Console, System.Runtime.CompilerServices as Runtime;
            [Runtime.MethodImplAttribute(Runtime.MethodImplOptions.NoInlining)]
            procedure Observe(Input: int);
            begin
              const First: int := 41;
              const Removed: int := 42;
              Console.WriteLine(Input);
            end procedure;
            begin
              Observe(41);
            end;
            end program;
            """, "--optimize");
        program.UseFullyOptimizedJit();
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 7)), deadline.Token);
        await host.Initialize(deadline.Token);
        var exited = await host.WaitForStop(deadline.Token);
        Assert.Equal("exited", exited.State); Assert.Null(exited.Error); Assert.Equal(0, exited.ExitCode);
        var rejected = Assert.Single(exited.Breakpoints!);
        Assert.Equal("rejected", rejected.State); Assert.Equal("unbound_breakpoint", rejected.Error!.Code);
        await host.Stop(deadline.Token);
        Assert.Contains("41", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Fact]
    public async Task Optimized_scalar_bindings_report_missing_storage_and_never_reuse_a_previous_value()
    {
        using var program = new Debuggee("own-optimized-values");
        var source = """
            program OptimizedScalars;
            uses System.Console as Console, System.IO.File as File, System.Runtime.CompilerServices as Runtime;
            [Runtime.MethodImplAttribute(Runtime.MethodImplOptions.NoInlining)]
            function Calculate(Input: int): int;
            begin
              const Unused: long := cast<long>(Input) + 100L;
              var Value: int := Input + 1;
              File.WriteAllText('first.txt', 'done');
              Value := Value + 1;
              File.WriteAllText('second.txt', 'second');
              return Value;
            end function;
            begin
              Console.WriteLine(Calculate(41));
            end;
            end program;
            """;
        await program.Compile(source, "--optimize");
        using (var image = new PEReader(File.OpenRead(program.Assembly)))
        {
            var metadata = image.GetMetadataReader();
            var method = metadata.MethodDefinitions.Select(metadata.GetMethodDefinition).Single(item => metadata.GetString(item.Name) == "Calculate");
            Assert.True((method.ImplAttributes & MethodImplAttributes.NoInlining) != 0);
        }
        program.UseFullyOptimizedJit();
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, source, deadline.Token);
        var before = host.Snapshot; var firstFrame = await Frame(host, before, deadline.Token);
        var first = await host.FrameBindings(new(before.StopId!.Value, firstFrame.Id), deadline.Token);
        Assert.False(File.Exists(program.FilePath("first.txt")));
        Assert.Contains(first.Items, item => item.Name == "Unused" && item.Status == "optimized_out" && item.Scalar is null);
        CheckValue(Assert.Single(first.Items, item => item.Name == "Value"), "42");
        await host.SetBreakpoints(new(program.Source, [new(10)], before.StopId), deadline.Token);
        await host.Resume(before.StopId.Value, "continue", deadline.Token);
        var after = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", after.Reason);
        var nextFrame = await Frame(host, after, deadline.Token);
        var next = await host.FrameBindings(new(after.StopId!.Value, nextFrame.Id), deadline.Token);
        Assert.Equal("done", File.ReadAllText(program.FilePath("first.txt")));
        Assert.False(File.Exists(program.FilePath("second.txt")));
        CheckValue(Assert.Single(next.Items, item => item.Name == "Value"), "43");
        var stale = await Assert.ThrowsAsync<DebugException>(() => host.FrameBindings(new(after.StopId.Value, firstFrame.Id), deadline.Token));
        Assert.Equal("stale_reference", stale.Code);
        await host.Resume(after.StopId.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        Assert.Equal("second", File.ReadAllText(program.FilePath("second.txt")));
        await host.Stop(deadline.Token);

        static void CheckValue(DebugBinding binding, string expected)
        {
            Assert.Contains(binding.Status, new[] { "available", "optimized_out" });
            if (binding.Status == "available") Assert.Equal(expected, binding.Scalar);
            else Assert.Null(binding.Scalar);
        }
    }

    [Fact]
    public async Task Passive_scalar_reads_preserve_widths_special_values_decimal_enum_and_utf16_content()
    {
        using var program = new Debuggee("own-passive-scalars");
        var source = """
            program Scalars;
            uses System.Console as Console, System.Double as Real, System.Single as Single, System.String as Text;
            type Choice = enum : ulong Last = 18446744073709551615UL; end enum;
            procedure Observe();
            begin
              const Boolean: bool := true;
              const SignedByte: sbyte := -128;
              const UnsignedByte: byte := 255;
              const SignedShort: short := -32768;
              const UnsignedShort: ushort := 65535;
              const SignedInt: int := -2147483648;
              const UnsignedInt: uint := 4294967295U;
              const SignedLong: long := -9223372036854775808L;
              const UnsignedLong: ulong := 18446744073709551615UL;
              const NativeSigned: nint := nint.MinValue;
              const NativeUnsigned: nuint := nuint.MaxValue;
              const FiniteSingle: float := 1.25F;
              const FiniteDouble: double := -1.125;
              const NotNumber: double := Real.NaN;
              const Positive: double := Real.PositiveInfinity;
              const Negative: float := Single.NegativeInfinity;
              const NegativeZero: double := -0.0;
              const Amount: decimal := 1.2500M;
              const Maximum: decimal := decimal.MaxValue;
              const Minimum: decimal := decimal.MinValue;
              const Selected: Choice := Choice.Last;
              const Character: char := #955;
              const Surrogate: char := #55296;
              const Unicode: string := 'λ😀' + #10 + #0 + '"\';
              const Lone: string := '' + #55296;
              const LongText: string := new Text(#955, 5000);
              const Missing: string? := null;
              Console.WriteLine('done');
            end procedure;
            begin
              Observe();
            end;
            end program;
            """;
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, source, deadline.Token);
        var stopped = host.Snapshot; var frame = await Frame(host, stopped, deadline.Token);
        var page = await host.FrameBindings(new(stopped.StopId!.Value, frame.Id, Count: 128), deadline.Token);
        Assert.False(page.Truncated); Assert.Equal(27, page.Total);
        var values = page.Items.ToDictionary(value => value.Name);
        Assert.All(values.Values, value => Assert.Equal("available", value.Status));
        var expected = new Dictionary<string, string>
        {
            ["Boolean"] = "true", ["SignedByte"] = "-128", ["UnsignedByte"] = "255", ["SignedShort"] = "-32768", ["UnsignedShort"] = "65535",
            ["SignedInt"] = "-2147483648", ["UnsignedInt"] = "4294967295", ["SignedLong"] = "-9223372036854775808", ["UnsignedLong"] = "18446744073709551615",
            ["NativeSigned"] = "-9223372036854775808", ["NativeUnsigned"] = "18446744073709551615", ["FiniteSingle"] = "1.25", ["FiniteDouble"] = "-1.125",
            ["NotNumber"] = "NaN", ["Positive"] = "Infinity", ["Negative"] = "-Infinity", ["NegativeZero"] = "-0", ["Amount"] = "1.2500",
            ["Maximum"] = "79228162514264337593543950335", ["Minimum"] = "-79228162514264337593543950335", ["Selected"] = "18446744073709551615", ["Character"] = "λ"
        };
        foreach (var (name, scalar) in expected) Assert.Equal(scalar, values[name].Scalar);
        Assert.Equal("enum", values["Selected"].Kind); Assert.EndsWith("+Choice", values["Selected"].Type);
        Assert.Equal("λ😀\n\0\"\\", values["Unicode"].Scalar);
        Assert.Equal("D800", values["Surrogate"].RawUtf16); Assert.Equal("D800", values["Lone"].RawUtf16);
        Assert.Equal("�", values["Lone"].Scalar); Assert.Equal("�", values["Surrogate"].Scalar);
        Assert.Equal(new string('λ', 4096), values["LongText"].Scalar); Assert.True(values["LongText"].Truncated);
        Assert.Equal("null", values["Missing"].Kind); Assert.Null(values["Missing"].Scalar);
        await host.Resume(stopped.StopId.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        var stale = await Assert.ThrowsAsync<DebugException>(() => host.FrameBindings(new(stopped.StopId.Value, frame.Id), deadline.Token));
        Assert.Equal("stale_reference", stale.Code);
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task Binding_pages_bound_large_strings_without_losing_the_remaining_binding_catalog()
    {
        using var program = new Debuggee("own-binding-pages");
        var declarations = string.Join('\n', Enumerable.Range(0, 40).Select(index => $"  const Text{index}: string := Text;"));
        var source = $$"""
            program BindingPages;
            uses System.Console as Console, System.String as String;
            procedure Observe();
            begin
              const Text: string := new String(#955, 5000);
            {{declarations}}
              Console.WriteLine('done');
            end procedure;
            begin
              Observe();
            end;
            end program;
            """;
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, source, deadline.Token);
        var stop = host.Snapshot; var frame = await Frame(host, stop, deadline.Token);
        var first = await host.FrameBindings(new(stop.StopId!.Value, frame.Id, Count: 128), deadline.Token);
        Assert.Equal(41, first.Total); Assert.True(first.Truncated); Assert.InRange(first.Items.Length, 1, 40);
        var values = new List<DebugBinding>(first.Items);
        while (values.Count < first.Total)
        {
            var next = await host.FrameBindings(new(stop.StopId.Value, frame.Id, values.Count, 128), deadline.Token);
            Assert.Equal(first.Total, next.Total); Assert.NotEmpty(next.Items); values.AddRange(next.Items);
        }
        Assert.Equal(41, values.Select(value => value.Name).Distinct().Count());
        Assert.All(values, value => { Assert.Equal(new string('λ', 4096), value.Scalar); Assert.True(value.Truncated); });
        var empty = await host.FrameBindings(new(stop.StopId.Value, frame.Id, 41, 1), deadline.Token);
        Assert.Empty(empty.Items); Assert.Equal(41, empty.Total); Assert.False(empty.Truncated);
        foreach (var request in new[] { new FrameRequest(stop.StopId.Value, frame.Id, -1, 1), new(stop.StopId.Value, frame.Id, 0, 0), new(stop.StopId.Value, frame.Id, 0, 129) })
        {
            var invalid = await Assert.ThrowsAsync<DebugException>(() => host.FrameBindings(request, deadline.Token));
            Assert.Equal("invalid_argument", invalid.Code);
        }
        Assert.Equal(stop.StopId, host.Snapshot.StopId);
        await host.Stop(deadline.Token);
    }

    private static async Task<DebugSessionHost> Launch(Debuggee program, string source, CancellationToken token)
    {
        var line = Array.FindIndex(source.Split('\n'), text => text.Contains("Console.WriteLine('done')", StringComparison.Ordinal) || text.Contains("File.WriteAllText('first.txt'", StringComparison.Ordinal)) + 1;
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, token);
        try
        {
            await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, line)), token);
            await host.Initialize(token);
            var stopped = await host.WaitForStop(token);
            Assert.True(stopped.Reason == "breakpoint", $"Expected breakpoint, got {stopped.State}/{stopped.Reason}: {stopped.Error}; {string.Join(',', stopped.Breakpoints?.Select(point => point.Error?.Message ?? point.State) ?? [])}");
            return host;
        }
        catch { await host.DisposeAsync(); throw; }
    }

    private static async Task<DebugFrame> Frame(DebugSessionHost host, DebugSnapshot stop, CancellationToken token)
    {
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, token), item => item.IsStopThread);
        return (await host.Stack(new(stop.StopId.Value, thread.Id), token)).Items[0];
    }

}
