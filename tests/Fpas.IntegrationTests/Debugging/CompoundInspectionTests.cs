using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class CompoundInspectionTests
{
    [Fact]
    public async Task Raw_objects_and_arrays_preserve_type_identity_cycles_bounds_and_stop_lifetimes_without_evaluation()
    {
        using var program = new Debuggee("own-compound-values");
        var library = await Library(program);
        var source = """
            program Compound;
            uses Fixture as F, System.Console as Console;
            procedure Observe();
            begin
              const Root: F.Node<int> := new F.Node<int>(42);
              Root.Next := Root;
              const Alias: F.Node<int> := Root;
              const Values: array of int := [10, 20, 30];
              const Matrix: array[,] of int := new int[2, 3];
              Matrix[1, 2] := 42;
              const Offset: array[,] of int := F.Arrays.Offset();
              const Boxed: object := 42; const Chain: F.Node<int> := F.Arrays.Chain(12); const Native: object := F.Arrays.Pointer();
              Console.WriteLine('observe');
              Root.Value := 43;
              Values[0] := 11;
              Console.WriteLine('after');
            end procedure;
            begin
              Observe();
            end;
            end program;
            """;
        await program.Compile(source, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 13)), deadline.Token);
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        var failure = stop.Reason == "exception" ? await host.ExceptionInfo(stop.StopId!.Value, deadline.Token) : null;
        Assert.True(stop.Reason == "breakpoint", $"Expected breakpoint, got {stop.State}/{stop.Reason}: {failure?.Type}: {failure?.Message.Text}; {stop.Error}");
        var frame = await Frame(host, stop, deadline.Token);
        var bindings = await host.FrameBindings(new(stop.StopId!.Value, frame.Id), deadline.Token);
        Assert.All(bindings.Items, item => Assert.Equal("available", item.Status));
        var values = bindings.Items.ToDictionary(item => item.Name);
        var root = values["Root"]; var alias = values["Alias"];
        Assert.NotNull(root.Reference); Assert.Equal(root.ObjectId, alias.ObjectId); Assert.NotEqual(root.Reference, alias.Reference);
        Assert.Equal("int", Assert.Single(root.RuntimeType!.Arguments!).Name);
        Assert.NotNull(root.RuntimeType.ModuleId); Assert.NotNull(root.RuntimeType.ModuleIdentity); Assert.NotNull(root.RuntimeType.Token);
        var repeated = await host.FrameBindings(new(stop.StopId.Value, frame.Id), deadline.Token);
        Assert.Equal(root.Reference, Assert.Single(repeated.Items, item => item.Name == "Root").Reference);
        var fields = await host.ValueChildren(new(stop.StopId.Value, root.Reference.Value), deadline.Token);
        Assert.Equal("42", Assert.Single(fields.Items, item => item.Name == "Value").Scalar);
        var sourceFields = await host.ValueChildren(new(stop.StopId.Value, root.Reference.Value, View: "source"), deadline.Token);
        var getter = Assert.Single(sourceFields.Items, item => item.Name == "Dangerous");
        Assert.Equal("unavailable", getter.Status); Assert.Equal("requires_execution", getter.Error!.Code);
        foreach (var (expression, expected) in new[] { ("Root.Value + 1", "43"), ("Offset[-1, 7]", "99") })
        {
            var evaluated = await host.Evaluate(new(stop.StopId.Value, frame.Id, expression), deadline.Token);
            Assert.Null(evaluated.Error); Assert.Equal(expected, evaluated.Value?.Scalar);
        }
        var rejectedGetter = await host.Evaluate(new(stop.StopId.Value, frame.Id, "Root.Dangerous"), deadline.Token);
        Assert.Equal("requires_execution", rejectedGetter.Error?.Code);
        Assert.Equal("unsupported_expression", (await host.Evaluate(new(stop.StopId.Value, frame.Id, "Root + Root"), deadline.Token)).Error?.Code);
        var cycle = Assert.Single(fields.Items, item => item.Name == "Next");
        Assert.True(cycle.Cycle); Assert.Equal(root.ObjectId, cycle.ObjectId); Assert.Null(cycle.Reference);
        var vector = await host.ValueChildren(new(stop.StopId.Value, values["Values"].Reference!.Value, 1, 2), deadline.Token);
        Assert.Equal(3, vector.Total); Assert.Equal(new[] { "[1]", "[2]" }, vector.Items.Select(item => item.Name));
        Assert.Equal(new[] { "20", "30" }, vector.Items.Select(item => item.Scalar));
        Assert.Equal(new[] { 2, 3 }, values["Matrix"].Lengths); Assert.Equal(new[] { 0, 0 }, values["Matrix"].LowerBounds);
        var matrix = await host.ValueChildren(new(stop.StopId.Value, values["Matrix"].Reference!.Value, 4, 2), deadline.Token);
        Assert.Equal(new[] { "[1,1]", "[1,2]" }, matrix.Items.Select(item => item.Name));
        Assert.Equal(new[] { "0", "42" }, matrix.Items.Select(item => item.Scalar));
        Assert.Equal(new[] { -2, 5 }, values["Offset"].LowerBounds);
        var offset = await host.ValueChildren(new(stop.StopId.Value, values["Offset"].Reference!.Value, 5, 1), deadline.Token);
        Assert.Equal("[-1,7]", Assert.Single(offset.Items).Name); Assert.Equal("99", Assert.Single(offset.Items).Scalar);
        var boxed = await host.ValueChildren(new(stop.StopId.Value, values["Boxed"].Reference!.Value), deadline.Token);
        Assert.Equal("42", Assert.Single(boxed.Items).Scalar);
        var opaque = Assert.Single((await host.ValueChildren(new(stop.StopId.Value, values["Native"].Reference!.Value), deadline.Token)).Items);
        Assert.Equal("unavailable", opaque.Status); Assert.Equal("unsupported_value", opaque.Error!.Code); Assert.Null(opaque.Reference);
        var chain = values["Chain"];
        for (var depth = 0; depth < 8; depth++)
        {
            var segment = await host.ValueChildren(new(stop.StopId.Value, chain.Reference!.Value), deadline.Token);
            Assert.Equal(depth.ToString(System.Globalization.CultureInfo.InvariantCulture), Assert.Single(segment.Items, item => item.Name == "Value").Scalar);
            chain = Assert.Single(segment.Items, item => item.Name == "Next");
        }
        Assert.True(chain.Truncated); Assert.False(chain.Cycle); Assert.Null(chain.Reference); Assert.Equal(2, chain.Children);
        foreach (var sentinel in new[] { "getter.txt", "display.txt", "proxy.txt", "operator.txt" }) Assert.False(File.Exists(program.FilePath(sentinel)));
        await host.SetBreakpoints(new(program.Source, [new(16)], stop.StopId), deadline.Token);
        await host.Resume(stop.StopId.Value, "continue", deadline.Token);
        var next = await host.WaitForStop(deadline.Token); Assert.Equal("breakpoint", next.Reason);
        var stale = await Assert.ThrowsAsync<DebugException>(() => host.ValueChildren(new(next.StopId!.Value, root.Reference.Value), deadline.Token));
        Assert.Equal("stale_reference", stale.Code);
        var nextFrame = await Frame(host, next, deadline.Token);
        var current = (await host.FrameBindings(new(next.StopId!.Value, nextFrame.Id), deadline.Token)).Items.ToDictionary(item => item.Name);
        Assert.NotEqual(root.Reference, current["Root"].Reference);
        var updated = await host.ValueChildren(new(next.StopId.Value, current["Root"].Reference!.Value), deadline.Token);
        Assert.Equal("43", Assert.Single(updated.Items, item => item.Name == "Value").Scalar);
        var changed = await host.ValueChildren(new(next.StopId.Value, current["Values"].Reference!.Value, 0, 1), deadline.Token);
        Assert.Equal("11", Assert.Single(changed.Items).Scalar);
        await host.Resume(next.StopId.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        foreach (var sentinel in new[] { "getter.txt", "display.txt", "proxy.txt", "operator.txt" }) Assert.False(File.Exists(program.FilePath(sentinel)));
    }

    [Fact]
    public async Task Value_reference_limits_remain_local_and_existing_paths_stay_readable()
    {
        using var program = new Debuggee("own-value-limits");
        var library = await Library(program);
        await program.Compile("""
            program ValueLimits;
            uses Fixture as F, System.Console as Console;
            procedure Observe();
            begin
              const Pool: array of F.Node<int> := F.Arrays.Many(5000);
              Console.WriteLine('observe');
            end procedure;
            begin Observe(); end;
            end program;
            """, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(60));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 6)), deadline.Token);
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token); Assert.Equal("breakpoint", stop.Reason);
        var frame = await Frame(host, stop, deadline.Token);
        var pool = Assert.Single((await host.FrameBindings(new(stop.StopId!.Value, frame.Id), deadline.Token)).Items);
        var available = 0; var limited = 0; long? first = null;
        for (var start = 0; start < 5000; start += 128)
        {
            var page = await host.ValueChildren(new(stop.StopId.Value, pool.Reference!.Value, start, Math.Min(128, 5000 - start)), deadline.Token);
            Assert.Equal(5000, page.Total); Assert.False(page.Truncated);
            foreach (var value in page.Items)
            {
                if (value.Status == "available") { available++; Assert.NotNull(value.Reference); first ??= value.Reference; }
                else { limited++; Assert.Equal("limit_exceeded", value.Error!.Code); Assert.Null(value.Reference); }
            }
        }
        Assert.Equal(4095, available); Assert.Equal(905, limited);
        var retained = await host.ValueChildren(new(stop.StopId.Value, first!.Value), deadline.Token);
        Assert.Equal("0", Assert.Single(retained.Items, item => item.Name == "Value").Scalar);
        foreach (var request in new[] { new ValueRequest(stop.StopId.Value, pool.Reference!.Value, -1, 1), new(stop.StopId.Value, pool.Reference.Value, 0, 129) })
        {
            var invalid = await Assert.ThrowsAsync<DebugException>(() => host.ValueChildren(request, deadline.Token)); Assert.Equal("invalid_argument", invalid.Code);
        }
        Assert.Equal(stop.StopId, host.Snapshot.StopId); await host.Stop(deadline.Token);
    }

    private static async Task<DebugFrame> Frame(DebugSessionHost host, DebugSnapshot stop, CancellationToken token)
    {
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, token), item => item.IsStopThread);
        return (await host.Stack(new(stop.StopId.Value, thread.Id), token)).Items[0];
    }

    private static async Task<string> Library(Debuggee program)
    {
        var project = program.FilePath("Fixture.csproj");
        File.WriteAllText(project, "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><AllowUnsafeBlocks>true</AllowUnsafeBlocks><NuGetAudit>false</NuGetAudit></PropertyGroup></Project>");
        File.WriteAllText(program.FilePath("Fixture.cs"), """
            using System;
            using System.Diagnostics;
            using System.IO;
            namespace Fixture;
            [DebuggerDisplay("{Dangerous}")]
            [DebuggerTypeProxy(typeof(Proxy))]
            public sealed class Node<T>(T value)
            {
                public T Value = value;
                public Node<T>? Next;
                public int Dangerous { get { File.WriteAllText("getter.txt", "bad"); return 1; } }
                public override string ToString() { File.WriteAllText("display.txt", "bad"); return "bad"; }
                public static Node<T> operator +(Node<T> left, Node<T> right) { File.WriteAllText("operator.txt", "bad"); return left; }
            }
            public sealed class Proxy
            {
                public Proxy(object value) { File.WriteAllText("proxy.txt", "bad"); }
            }
            public sealed unsafe class PointerBox { public int* Address = (int*)1; }
            public static class Arrays
            {
                public static object Pointer() => new PointerBox();
                public static Node<int> Chain(int count)
                {
                    var root = new Node<int>(0); var next = root;
                    for (int index = 1; index < count; index++) { next.Next = new Node<int>(index); next = next.Next; }
                    return root;
                }
                public static Node<int>[] Many(int count)
                {
                    var result = new Node<int>[count];
                    for (int index = 0; index < count; index++) result[index] = new Node<int>(index);
                    return result;
                }
                public static int[,] Offset()
                {
                    var result = (int[,])Array.CreateInstance(typeof(int), [2, 3], [-2, 5]);
                    result[-1, 7] = 99;
                    return result;
                }
            }
            """);
        var build = await ProcessRunner.RunAsync(["build", project, "--configuration", "Release", "--nologo", "--verbosity", "quiet"], workingDirectory: program.DirectoryPath);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var library = program.FilePath("Fixture.dll");
        File.Copy(program.FilePath("bin/Release/net10.0/Fixture.dll"), library); return library;
    }
}
