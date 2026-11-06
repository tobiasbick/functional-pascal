using System.Text.Json.Nodes;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Theory]
    [InlineData("Content-Length: 1048577\r\n\r\n")]
    [InlineData("Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}")]
    [InlineData("Content-Length: 1\r\n\r\n{")]
    public async Task Malformed_or_oversized_frames_fail_boundedly_and_reap_an_already_running_target(string frame)
    {
        using var program = new Debuggee("dap-invalid-frame"); await WaitingProgram(program);
        await using var client = new DapProbe(); await LaunchRunning(client, program);
        using var target = Track(await Ready(program));
        await client.Raw(frame); await client.End(1); Gone(target);
    }

    [Fact]
    public async Task Slow_client_receives_explicit_output_loss_before_terminal_events()
    {
        using var program = new Debuggee("dap-backpressure");
        await program.Compile($$"""
            program Flood;
            uses System.IO.File as File, System.Threading.Thread as Thread, System.Console as Console;
            begin
              while not File.Exists('go') do Thread.Sleep(5); end while;
              for Index: int := 1 to 8192 do Console.Write('{{new string('x', 1024)}}'); end for;
            end;
            end program;
            """);
        await using var client = new DapProbe(); await LaunchRunning(client, program);
        client.PauseReading(); File.WriteAllText(program.FilePath("go"), "go");
        await Task.Delay(400); client.ResumeReading();
        var events = await client.UntilTerminated();
        Assert.Contains(events, item => item.GetProperty("event").GetString() == "output" && item.GetProperty("body").GetProperty("output").GetString()!.Contains("output omitted", StringComparison.Ordinal));
        Assert.Contains(events, item => item.GetProperty("event").GetString() == "exited");
        await client.Success("disconnect"); await client.End();
    }

    [Fact]
    public async Task Stepping_another_live_thread_is_rejected_without_resuming_the_stop_thread()
    {
        using var program = new Debuggee("dap-other-thread");
        await program.Compile("""
            program Threads;
            uses System.Threading.ManualResetEventSlim as Event, System.Threading.Thread as Thread, System.Threading.Tasks as Tasks, System.Console as Console;
            begin
              const Ready: Event := new Event(false);
              const Worker: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int
              begin Ready.Set(); Thread.Sleep(60000); return 0; end function);
              Ready.Wait();
              Console.WriteLine('observe');
              discard Worker;
            end;
            end program;
            """);
        await using var client = new DapProbe(); await client.Success("initialize");
        await client.Success("launch", new() { ["program"] = program.Assembly }); await client.Event("initialized");
        await client.Success("setBreakpoints", new() { ["source"] = new JsonObject { ["path"] = program.Source }, ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = 8 }) });
        await client.Success("configurationDone"); var stop = await client.Event("stopped");
        var selected = stop.GetProperty("threadId").GetInt32();
        var threads = (await client.Success("threads")).GetProperty("threads");
        var other = threads.EnumerateArray().First(item => item.GetProperty("id").GetInt32() != selected).GetProperty("id").GetInt32();
        var rejected = await client.Request("stepIn", new() { ["threadId"] = other });
        Assert.False(rejected.GetProperty("success").GetBoolean()); Assert.Equal("unsupported_thread", rejected.GetProperty("message").GetString());
        var value = await client.Success("evaluate", new() { ["expression"] = "42" }); Assert.Equal("42", value.GetProperty("result").GetString());
        await client.Success("disconnect"); await client.End();
    }
}
