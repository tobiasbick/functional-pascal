using System.Diagnostics;
using System.Text;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class TargetIoTests
{
    private static WorkerLauncher Launcher => WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task An_exact_limit_distinguishes_a_valid_continuation_from_invalid_eof(bool completeRune)
    {
        using var program = new Debuggee("own-exact-utf8-boundary");
        await program.Compile($$"""
            program BoundaryOutput;
            uses System.Console as Console, System.IO.Stream as Stream, System.Threading.Thread as Thread;
            begin
              const Output: Stream := Console.OpenStandardOutput();
              Output.WriteByte(226); Output.WriteByte(130); Output.Flush(); Thread.Sleep(100);
              {{(completeRune ? "Output.WriteByte(172); Output.Flush();" : "")}}
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token, outputLimit: 2, retainOutputPrefix: true);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        await host.Initialize(deadline.Token); var entry = await host.WaitForStop(deadline.Token);
        await host.Resume(entry.StopId!.Value, "continue", deadline.Token); Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode);
        await host.Stop(deadline.Token); var output = host.Output();
        Assert.Equal(completeRune ? 3 : 2, output.TotalBytes); Assert.Equal(completeRune ? 0 : 2, output.RetainedBytes);
        Assert.Equal(completeRune ? "" : "\uFFFD", string.Concat(output.Items.Select(item => item.Text)));
        Assert.Equal(!completeRune, output.EncodingLoss); Assert.True(output.LimitReached); Assert.Equal(completeRune, output.Truncated);
    }

    [Theory]
    [InlineData(false, 64)]
    [InlineData(true, 64)]
    [InlineData(false, 3)]
    public async Task Utf8_sequences_split_across_reads_and_incomplete_eof_have_explicit_loss(bool incompleteTail, int limit)
    {
        using var program = new Debuggee("own-split-utf8-output");
        await program.Compile($$"""
            program SplitOutput;
            uses System.Console as Console, System.IO.Stream as Stream, System.Threading.Thread as Thread;
            begin
              const Output: Stream := Console.OpenStandardOutput();
              Output.WriteByte(240); Output.WriteByte(159); Output.Flush(); Thread.Sleep(100);
              Output.WriteByte(152); Output.WriteByte(128); Output.Flush();
              {{(incompleteTail ? "Output.WriteByte(226); Output.WriteByte(130); Output.Flush();" : "")}}
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token, outputLimit: limit, retainOutputPrefix: true);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        await host.Initialize(deadline.Token); var entry = await host.WaitForStop(deadline.Token);
        await host.Resume(entry.StopId!.Value, "continue", deadline.Token); Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode);
        await host.Stop(deadline.Token); var output = host.Output();
        Assert.Equal(incompleteTail ? 6 : 4, output.TotalBytes); Assert.Equal(limit == 3 ? 0 : incompleteTail ? 6 : 4, output.RetainedBytes);
        Assert.Equal(limit == 3 ? "" : incompleteTail ? "😀\uFFFD" : "😀", string.Concat(output.Items.Select(item => item.Text)));
        Assert.Equal(incompleteTail, output.EncodingLoss); Assert.Equal(limit == 3, output.LimitReached); Assert.Equal(limit == 3, output.Truncated);
    }

    [Fact]
    public async Task Output_limits_count_original_bytes_before_utf8_replacement()
    {
        using var program = new Debuggee("own-invalid-output-byte");
        await program.Compile("""
            program InvalidOutput;
            uses System.Console as Console, System.IO.Stream as Stream;
            begin
              const Output: Stream := Console.OpenStandardOutput();
              Output.WriteByte(255);
              Output.Flush();
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token, outputLimit: 3, retainOutputPrefix: true);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        await host.Initialize(deadline.Token); var entry = await host.WaitForStop(deadline.Token);
        await host.Resume(entry.StopId!.Value, "continue", deadline.Token); Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode);
        await host.Stop(deadline.Token); var output = host.Output();
        Assert.Equal(1, output.TotalBytes); Assert.Equal(1, output.RetainedBytes); Assert.False(output.LimitReached);
        Assert.Equal("\uFFFD", string.Concat(output.Items.Select(item => item.Text)));
        Assert.True(output.EncodingLoss);
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Target_arguments_preserve_elements_and_explicit_input_arrives_once_before_eof(bool suppliedInput)
    {
        using var program = new Debuggee(OperatingSystem.IsLinux() ? "own-target-io-λ \"quoted\\" : "own-target-io-λ space");
        await program.Compile("""
            program TargetIo;
            uses System.Console as Console, System.Convert as Convert, System.Text.Encoding as Text, System.IO.Stream as Stream;
            begin
              Console.OutputEncoding := Text.UTF8;
              Console.WriteLine(args.Length);
              for Index: int := 0 to args.Length - 1 do
                Console.WriteLine(Convert.ToBase64String(Text.UTF8.GetBytes(args[Index])));
              end for;
              const Input: Stream := Console.OpenStandardInput();
              var Count: int := 0;
              var Sum: long := 0L;
              var Next: int := Input.ReadByte();
              while Next >= 0 do
                Count := Count + 1;
                Sum := Sum + Next;
                Next := Input.ReadByte();
              end while;
              Console.WriteLine(Count);
              Console.WriteLine(Sum);
            end;
            end program;
            """);
        var arguments = new[] { "", "simple", "two words", "λ😀", "a\"b", "trailing\\", "--option", "$() ; &", "line\nbreak", "a\\\"b", "a\\\\\"b", "a\\\\\\\"b", "trailing\\\\", "trailing\\\\\\", "\\", "\\\\", "", "" };
        var input = Enumerable.Range(0, 1024).Select(index => (byte)(index % 256)).ToArray();
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token, suppliedInput ? input : null);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, Arguments: arguments), deadline.Token);
        await host.Initialize(deadline.Token); var entry = await host.WaitForStop(deadline.Token); Assert.Equal("entry", entry.Reason);
        await host.Resume(entry.StopId!.Value, "continue", deadline.Token); var exited = await host.WaitForStop(deadline.Token);
        Assert.Equal("exited", exited.State); Assert.Equal(0, exited.ExitCode); await host.Stop(deadline.Token);
        var lines = string.Concat(host.Output().Items.Select(item => item.Text)).Replace("\r\n", "\n", StringComparison.Ordinal).TrimEnd('\n').Split('\n');
        Assert.Equal(suppliedInput ? "1024" : "0", lines[^2]);
        Assert.Equal((suppliedInput ? input.Sum(value => (long)value) : 0).ToString(System.Globalization.CultureInfo.InvariantCulture), lines[^1]);
        Assert.Equal(arguments.Select(argument => Convert.ToBase64String(Encoding.UTF8.GetBytes(argument))), lines.Skip(1).Take(int.Parse(lines[0], System.Globalization.CultureInfo.InvariantCulture)));
        Assert.Equal(arguments.Length.ToString(System.Globalization.CultureInfo.InvariantCulture), lines[0]);
    }

    [Fact]
    public async Task Unrepresentable_dotnet_executable_paths_fail_before_a_target_is_created()
    {
        using var program = new Debuggee("own-dotnet-host-path");
        await program.Compile("program HostPath; begin end; end program;");
        string dotnet;
        if (OperatingSystem.IsLinux())
        {
            var directory = program.FilePath("quoted\" host");
            Directory.CreateSymbolicLink(directory, Path.GetDirectoryName(BuildProcess.DotnetHost())!);
            dotnet = Path.Combine(directory, Path.GetFileName(BuildProcess.DotnetHost())); Assert.True(File.Exists(dotnet));
        }
        else dotnet = program.FilePath("unrepresentable\"host.exe");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token);
        var failure = await Assert.ThrowsAsync<DebugException>(() => host.Prepare(new(dotnet, program.Assembly, program.DirectoryPath), deadline.Token));
        Assert.Equal("invalid_argument", failure.Code); Assert.Null(host.Snapshot.ProcessId); await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task A_blocked_input_writer_does_not_block_pause_or_owned_shutdown()
    {
        using var program = new Debuggee("own-blocked-target-input");
        await program.Compile("""
            program BlockedInput;
            uses System.Threading.Thread as Thread;
            begin
              while true do Thread.Sleep(100); end while;
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token, new byte[1_048_576]);
        var starting = await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        var target = starting.ProcessId!.Value; var worker = host.WorkerProcessId;
        await host.Initialize(deadline.Token); var entry = await host.WaitForStop(deadline.Token);
        await host.Resume(entry.StopId!.Value, "continue", deadline.Token); await Task.Delay(100, deadline.Token);
        await host.Pause(deadline.Token); var paused = await host.WaitForStop(deadline.Token); Assert.Equal("pause", paused.Reason);
        await host.Stop(deadline.Token);
        AssertExited(target); AssertExited(worker);
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task A_prefix_capture_preserves_utf8_boundaries_and_signals_the_exact_byte_limit(bool exact)
    {
        using var program = new Debuggee("own-prefix-output");
        await program.Compile($$"""
            program PrefixOutput;
            uses System.Console as Console, System.Text.Encoding as Text, System.Threading.Thread as Thread;
            begin
              Console.OutputEncoding := Text.UTF8;
              {{(exact ? "Console.Write('abcdefg');" : "Console.Write('😀'); Console.Out.Flush(); Thread.Sleep(100); Console.Write('😀'); Console.Out.Flush(); Thread.Sleep(100); Console.Write('end');")}}
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, deadline.Token, outputLimit: 7, retainOutputPrefix: true);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        await host.Initialize(deadline.Token); var entry = await host.WaitForStop(deadline.Token);
        await host.Resume(entry.StopId!.Value, "continue", deadline.Token); Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode);
        await host.Stop(deadline.Token);
        var output = host.Output(); Assert.True(host.OutputLimitReached.IsCompleted); Assert.True(output.LimitReached);
        Assert.Equal(exact ? "abcdefg" : "😀", string.Concat(output.Items.Select(item => item.Text)));
        Assert.Equal(exact ? 7 : 11, output.TotalBytes); Assert.Equal(exact ? 7 : 4, output.RetainedBytes);
        Assert.Equal(!exact, output.Truncated);
        Assert.False(output.EncodingLoss);
    }

    private static void AssertExited(int id)
    {
        try { using var process = Process.GetProcessById(id); Assert.True(process.HasExited, "An owned I/O process survived shutdown."); }
        catch (ArgumentException) { }
    }
}
