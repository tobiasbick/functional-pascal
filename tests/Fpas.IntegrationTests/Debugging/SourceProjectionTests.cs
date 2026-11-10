using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class SourceProjectionTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Source_record_receivers_use_self_in_ordinary_and_async_frames(bool asynchronous)
    {
        using var program = new Debuggee("own-source-self");
        var source = $$"""
            program Receiver;
            uses System.Console as Console, System.Threading.Tasks as Tasks;
            type Counter = record
              public Value: int;
              {{(asynchronous ? "public async function Read(): Tasks.Task<int>;" : "public function Read(): int;")}}
              begin
                {{(asynchronous ? "await Tasks.Task.Delay(10);" : "discard 0;")}}
                Console.WriteLine('self');
                return self.Value;
              end function;
            end record;
            begin
              const Value: Counter := new Counter(42);
              Console.WriteLine(Value.Read(){{(asynchronous ? ".GetAwaiter().GetResult()" : "")}});
            end;
            end program;
            """;
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost()), program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 8)), deadline.Token);
        await host.Initialize(deadline.Token); var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", stop.Reason);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var frame = (await host.Stack(new(stop.StopId.Value, thread.Id), deadline.Token)).Items[0];
        var bindings = await host.FrameBindings(new(stop.StopId.Value, frame.Id), deadline.Token);
        var receiver = Assert.Single(bindings.Items, item => item.Name == "self"); Assert.Equal("record", receiver.Kind);
        Assert.Equal("42", Assert.Single((await host.ValueChildren(new(stop.StopId.Value, receiver.Reference!.Value, View: "source"), deadline.Token)).Items).Scalar);
        var raw = await host.FrameBindings(new(stop.StopId.Value, frame.Id, View: "raw"), deadline.Token);
        Assert.DoesNotContain(raw.Items, item => item.Name == "self");
        if (!asynchronous) Assert.Contains(raw.Items, item => item.Name == "this");
        await host.Resume(stop.StopId.Value, "continue", deadline.Token); Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode);
        await host.Stop(deadline.Token);
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Invalid_source_presentation_metadata_preserves_raw_storage_and_the_stopped_context(bool instanceFrame)
    {
        using var program = new Debuggee("own-invalid-source-projection");
        File.Copy(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!, "Fpas.Core.dll"), program.FilePath("Fpas.Core.dll"));
        File.WriteAllText(program.FilePath("Fixture.csproj"), "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><NuGetAudit>false</NuGetAudit></PropertyGroup><ItemGroup><Reference Include=\"Fpas.Core\"><HintPath>Fpas.Core.dll</HintPath></Reference></ItemGroup></Project>");
        var foreignSource = """
            namespace Fixture;
            [Fpas.Core.Domain(2, "System.Int32", null, null)]
            public sealed class Broken
            {
                public int Value = 42;
                public void Inspect()
                {
                    System.Console.WriteLine("broken method");
                }
            }
            public static class Factory
            {
                public static object Create() => new Broken();
                public static void Call() => new Broken().Inspect();
            }
            """;
        File.WriteAllText(program.FilePath("Fixture.cs"), foreignSource);
        var build = await ProcessRunner.RunAsync(["build", program.FilePath("Fixture.csproj"), "--nologo", "--verbosity", "quiet"], workingDirectory: program.DirectoryPath);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var library = program.FilePath("Fixture.dll"); File.Copy(program.FilePath("bin/Debug/net10.0/Fixture.dll"), library);
        File.Copy(program.FilePath("bin/Debug/net10.0/Fixture.pdb"), program.FilePath("Fixture.pdb"));
        File.Delete(program.FilePath("Fpas.Core.dll"));
        var source = """
            program BrokenPresentation;
            uses Fixture.Factory as Factory, System.Console as Console;
            begin
              const Value: object := Factory.Create();
              Factory.Call();
              Console.WriteLine('observe');
            end;
            end program;
            """;
        await program.Compile(source, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost()), program.DirectoryPath, deadline.Token);
        var location = instanceFrame ? new SourceBreakpoint(program.FilePath("Fixture.cs"), Array.FindIndex(foreignSource.Split('\n'), line => line.Contains("broken method", StringComparison.Ordinal)) + 1) : new(program.Source, 6);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, location), deadline.Token);
        await host.Initialize(deadline.Token); var stop = await host.WaitForStop(deadline.Token);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var frame = (await host.Stack(new(stop.StopId.Value, thread.Id), deadline.Token)).Items[0];
        var value = Assert.Single((await host.FrameBindings(new(stop.StopId.Value, frame.Id), deadline.Token)).Items, item => item.Name == (instanceFrame ? "this" : "Value"));
        Assert.Equal("available", value.Status); Assert.NotNull(value.Reference);
        Assert.Equal("invalid_metadata", value.SourceError!.Code);
        var raw = await host.ValueChildren(new(stop.StopId.Value, value.Reference!.Value), deadline.Token);
        Assert.Equal("42", Assert.Single(raw.Items).Scalar);
        var failed = await Assert.ThrowsAsync<DebugException>(() => host.ValueChildren(new(stop.StopId.Value, value.Reference.Value, View: "source"), deadline.Token));
        Assert.Equal("invalid_metadata", failed.Code); Assert.Equal(stop.Revision, host.Snapshot.Revision);
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task Fpas_records_variant_cases_and_domains_have_passive_source_and_raw_views()
    {
        using var program = new Debuggee("own-source-values");
        var source = """
            program SourceValues;
            uses System.Console as Console;
            type Point = record public X: int; public Label: string; end record;
            type Envelope<T> = record public Value: T; end record;
            type Choice<T> = variant Missing; Found(Value: T); end variant;
            type UserId = distinct ulong;
            type Amount = distinct decimal;
            type Label = distinct string;
            type Letter = distinct char;
            type Window = int range -10..10;
            type Full = long range -9223372036854775808L..9223372036854775807L;
            procedure Observe();
            begin
              const Position: Point := new Point(X := 42, Label := 'λ😀');
              const Wrapped: Envelope<UserId> := new Envelope<UserId>(new UserId(18446744073709551615UL));
              const Selected: Choice<int> := Choice<int>.Found(42);
              const Missing: Choice<int> := Choice<int>.Missing;
              const Quantity: Amount := new Amount(1.2500M);
              const Name: Label := new Label('mandelbrot');
              const Character: Letter := new Letter(#955);
              const Low: Window := new Window(-8);
              const Defaults: array of Window := new Window[1];
              const Maximum: Full := new Full(9223372036854775807L);
              Console.WriteLine('observe');
            end procedure;
            begin Observe(); end;
            end program;
            """;
        await program.Compile(source);
        var line = Array.FindIndex(source.Split('\n'), line => line.Contains("Console.WriteLine('observe')", StringComparison.Ordinal)) + 1;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost()), program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, line)), deadline.Token);
        await host.Initialize(deadline.Token); var stop = await host.WaitForStop(deadline.Token); Assert.Equal("breakpoint", stop.Reason);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, deadline.Token), item => item.IsStopThread);
        var frame = (await host.Stack(new(stop.StopId.Value, thread.Id), deadline.Token)).Items[0];
        var bindings = await host.FrameBindings(new(stop.StopId.Value, frame.Id), deadline.Token);
        var position = Binding("Position"); Assert.Equal("record", position.Kind); Assert.Equal(2, position.SourceChildren);
        var fields = await Children(position); Assert.Equal(new[] { "X", "Label" }, fields.Items.Select(item => item.Name));
        Assert.Equal("42", fields.Items[0].Scalar); Assert.Equal("λ😀", fields.Items[1].Scalar);
        var raw = await Children(position, "raw"); Assert.Equal(new[] { "<X>k__BackingField", "<Label>k__BackingField" }, raw.Items.Select(item => item.Name));
        var wrapped = Assert.Single((await Children(Binding("Wrapped"))).Items); Assert.Equal("domain", wrapped.Kind);
        Assert.Equal("18446744073709551615", Assert.Single((await Children(wrapped)).Items).Scalar);
        var selected = Binding("Selected"); Assert.Equal("variant", selected.Kind); Assert.Equal("Found", selected.Scalar);
        var payload = await Children(selected); Assert.Equal("1", Assert.Single(payload.Items, item => item.Name == "Tag").Scalar);
        Assert.Equal("42", Assert.Single(payload.Items, item => item.Name == "Value").Scalar);
        var missing = Binding("Missing"); Assert.Equal(0, missing.Children); Assert.Equal(1, missing.SourceChildren);
        Assert.Equal("Missing", missing.Scalar); Assert.Empty((await Children(missing, "raw")).Items);
        Assert.Equal("0", Assert.Single((await Children(missing)).Items).Scalar);
        Assert.Equal("1.2500", Assert.Single((await Children(Binding("Quantity"))).Items).Scalar);
        Assert.Equal("mandelbrot", Assert.Single((await Children(Binding("Name"))).Items).Scalar);
        Assert.Equal("λ", Assert.Single((await Children(Binding("Character"))).Items).Scalar);
        Assert.Equal("-8", Assert.Single((await Children(Binding("Low"))).Items).Scalar);
        Assert.Equal("2", Assert.Single((await Children(Binding("Low"), "raw")).Items).Scalar);
        var defaultValue = Assert.Single((await Children(Binding("Defaults"))).Items);
        Assert.Equal("-10", Assert.Single((await Children(defaultValue)).Items).Scalar);
        Assert.Equal("9223372036854775807", Assert.Single((await Children(Binding("Maximum"))).Items).Scalar);
        Assert.Equal("18446744073709551615", Assert.Single((await Children(Binding("Maximum"), "raw")).Items).Scalar);
        await host.Resume(stop.StopId.Value, "continue", deadline.Token); Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode);
        await host.Stop(deadline.Token);

        DebugBinding Binding(string name) => Assert.Single(bindings.Items, item => item.Name == name);
        Task<BindingPage> Children(DebugBinding value, string view = "source")
        { Assert.NotNull(value.Reference); return host.ValueChildren(new(stop.StopId.Value, value.Reference!.Value, View: view), deadline.Token); }
    }
}
