using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class StaticInspectionTests
{
    [Fact]
    public async Task Static_reads_preserve_generic_and_thread_context_without_initializing_dormant_types()
    {
        using var program = new Debuggee("own-static-values");
        var library = await Library(program);
        var source = """
            program Statics;
            uses Fixture as F, System.Console as Console;
            var Global: int := 21;
            procedure Observe();
            begin
              const Global: int := 99;
              const Integer: F.Slot<int> := new F.Slot<int>();
              const Text: F.Slot<string> := new F.Slot<string>(); const EmptyInt: F.EmptySlot<int> := new F.EmptySlot<int>(); const EmptyText: F.EmptySlot<string> := new F.EmptySlot<string>();
              Console.WriteLine('observe');
              F.Warm.Number := 43;
              Console.WriteLine('after');
            end procedure;
            begin
              F.Warm.Setup();
              F.Slot<int>.Call(Observe);
            end;
            end program;
            """;
        await program.Compile(source, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(40));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 9)), deadline.Token);
        await host.Initialize(deadline.Token); var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", stop.Reason);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var stack = await host.Stack(new(stop.StopId.Value, thread.Id), deadline.Token); var frame = stack.Items[0];
        var locals = await host.FrameBindings(new(stop.StopId.Value, frame.Id), deadline.Token);
        Assert.Equal("99", Assert.Single(locals.Items, item => item.Name == "Global").Scalar);
        var globals = await host.StaticBindings(new(stop.StopId.Value, frame.Id), deadline.Token);
        Assert.Equal("21", Assert.Single(globals.Items, item => item.Name == "Global").Scalar);
        var primitive = await host.StaticBindings(new(stop.StopId.Value, frame.Id, "System.Int32"), deadline.Token);
        Assert.Equal("2147483647", Assert.Single(primitive.Items, item => item.Name == "MaxValue").Scalar);
        var textStatics = await host.StaticBindings(new(stop.StopId.Value, frame.Id, "System.String"), deadline.Token);
        Assert.Equal("", Assert.Single(textStatics.Items, item => item.Name == "Empty").Scalar);
        var absent = await Assert.ThrowsAsync<DebugException>(() => host.StaticBindings(new(stop.StopId.Value, frame.Id, "Fixture.Absent"), deadline.Token));
        Assert.Equal("unavailable", absent.Code); Assert.Equal(stop.Revision, host.Snapshot.Revision);
        var genericDefinition = await Assert.ThrowsAsync<DebugException>(() => host.StaticBindings(new(stop.StopId.Value, frame.Id, "Fixture.Slot`1"), deadline.Token));
        Assert.Equal("unavailable", genericDefinition.Code); Assert.Equal(stop.Revision, host.Snapshot.Revision);
        var warm = await host.StaticBindings(new(stop.StopId.Value, frame.Id, "Fixture.Warm"), deadline.Token);
        Assert.Equal("42", Assert.Single(warm.Items, item => item.Name == "Number").Scalar);
        Assert.Equal("73", Assert.Single(warm.Items, item => item.Name == "ThreadValue").Scalar);
        Assert.Equal("18446744073709551615", Assert.Single(warm.Items, item => item.Name == "Maximum").Scalar);
        var literal = Assert.Single(warm.Items, item => item.Name == "Text");
        Assert.Equal("λ😀\uFFFD", literal.Scalar); Assert.Equal("03BBD83DDE00D800", literal.RawUtf16);
        var storage = Assert.Single(warm.Items, item => item.Name == "Values");
        Assert.Equal("7", Assert.Single((await host.ValueChildren(new(stop.StopId.Value, storage.Reference!.Value), deadline.Token)).Items).Scalar);
        var dormant = await host.StaticBindings(new(stop.StopId.Value, frame.Id, "Fixture.Dormant"), deadline.Token);
        Assert.Equal("unavailable", Assert.Single(dormant.Items).Status);
        Assert.False(File.Exists(program.FilePath("initialized.txt"))); Assert.False(File.Exists(program.FilePath("getter.txt")));
        foreach (var (name, expected) in new[] { ("Integer", "41"), ("Text", "99") })
        {
            var value = Assert.Single(locals.Items, item => item.Name == name);
            var fields = await host.ValueChildren(new(stop.StopId.Value, value.Reference!.Value, View: "static"), deadline.Token);
            Assert.Equal(expected, Assert.Single(fields.Items, item => item.Name == "Number").Scalar);
        }
        foreach (var (name, expected) in new[] { ("EmptyInt", "17"), ("EmptyText", "23") })
        {
            var value = Assert.Single(locals.Items, item => item.Name == name); Assert.Equal(0, value.Children); Assert.NotNull(value.Reference);
            var fields = await host.ValueChildren(new(stop.StopId.Value, value.Reference!.Value, View: "static"), deadline.Token);
            Assert.Equal(expected, Assert.Single(fields.Items, item => item.Name == "Number").Scalar);
        }
        var caller = Assert.Single(stack.Items, item => item.Name.Contains("Slot", StringComparison.Ordinal) && item.Name.Contains("Call", StringComparison.Ordinal));
        var callerStatics = await host.StaticBindings(new(stop.StopId.Value, caller.Id), deadline.Token);
        Assert.Equal("41", Assert.Single(callerStatics.Items, item => item.Name == "Number").Scalar);
        await host.SetBreakpoints(new(program.Source, [new(11)], stop.StopId), deadline.Token);
        await host.Resume(stop.StopId.Value, "continue", deadline.Token); var next = await host.WaitForStop(deadline.Token);
        var nextThread = Assert.Single(await host.Threads(next.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var nextFrame = (await host.Stack(new(next.StopId.Value, nextThread.Id), deadline.Token)).Items[0];
        var fresh = await host.StaticBindings(new(next.StopId.Value, nextFrame.Id, "Fixture.Warm"), deadline.Token);
        Assert.Equal("43", Assert.Single(fresh.Items, item => item.Name == "Number").Scalar);
        var stale = await Assert.ThrowsAsync<DebugException>(() => host.ValueChildren(new(next.StopId.Value, storage.Reference!.Value), deadline.Token));
        Assert.Equal("stale_reference", stale.Code);
        await host.Stop(deadline.Token);
        Assert.False(File.Exists(program.FilePath("initialized.txt"))); Assert.False(File.Exists(program.FilePath("getter.txt")));
    }

    private static async Task<string> Library(Debuggee program)
    {
        var project = program.FilePath("Fixture.csproj");
        File.WriteAllText(project, "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><NuGetAudit>false</NuGetAudit></PropertyGroup></Project>");
        File.WriteAllText(program.FilePath("Fixture.cs"), """
            using System;
            using System.IO;
            namespace Fixture;
            public sealed class Slot<T>
            {
                public T? Value;
                public static int Number;
                public static void Call(Action callback) { callback(); }
            }
            public sealed class EmptySlot<T> { public static int Number; }
            public static class Warm
            {
                public static int Number;
                [ThreadStatic] public static int ThreadValue;
                public const ulong Maximum = ulong.MaxValue;
                public const string Text = "λ😀\ud800";
                public static int[] Values = [];
                public static int Dangerous { get { File.WriteAllText("getter.txt", "bad"); return 1; } }
                public static void Setup() { Number = 42; ThreadValue = 73; Values = [7]; Slot<int>.Number = 41; Slot<string>.Number = 99; EmptySlot<int>.Number = 17; EmptySlot<string>.Number = 23; }
            }
            public static class Dormant
            {
                public static int Value;
                static Dormant() { File.WriteAllText("initialized.txt", "bad"); Value = 123; }
            }
            """);
        var build = await ProcessRunner.RunAsync(["build", project, "--configuration", "Debug", "--nologo", "--verbosity", "quiet"], workingDirectory: program.DirectoryPath);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var library = program.FilePath("Fixture.dll");
        File.Copy(program.FilePath("bin/Debug/net10.0/Fixture.dll"), library);
        File.Copy(program.FilePath("bin/Debug/net10.0/Fixture.pdb"), program.FilePath("Fixture.pdb"));
        return library;
    }
}
