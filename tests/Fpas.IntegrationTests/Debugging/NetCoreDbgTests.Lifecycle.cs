namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Pause_interrupts_running_user_code_and_disconnect_terminates_it()
    {
        using var program = new Debuggee("pause");
        await program.Compile("""
            program PauseProbe;
            uses System.Console as Console;
            begin
              var Count: long := 0L;
              while Count < 1000000000000L do
                Count := Count + 1L;
              end while;
              Console.WriteLine(Count);
            end;
            end program;
            """);
        await using var dap = new NetCoreDbgSession(program);
        var entry = await dap.Launch(program, 6);
        await dap.Continue(entry.Thread);
        var first = await dap.Stop();
        await dap.Breakpoints(program.Source);
        await dap.Continue(first.Thread);
        await dap.Request("pause", new { threadId = first.Thread });
        var stop = await dap.Stop("pause");
        Assert.Equal(program.Source, stop.Frame.GetProperty("source").GetProperty("path").GetString());
        Assert.Contains(stop.Frame.GetProperty("line").GetInt32(), new[] { 5, 6 });
        var count = Value(await dap.Locals(stop.Frame), "Count");
        Assert.True(long.Parse(count.GetProperty("value").GetString()!, System.Globalization.CultureInfo.InvariantCulture) >= 0);
        await dap.Disconnect();
    }

    [NetCoreDbgFact]
    public async Task Failed_launch_reports_an_error_and_the_adapter_can_disconnect_without_a_target()
    {
        using var program = new Debuggee("failed-launch");
        await using var dap = new NetCoreDbgSession(program);
        await dap.Request("initialize", new { adapterID = "coreclr" });
        await dap.Event("initialized");
        await dap.Request("launch", new { program = program.FilePath("missing-program"), cwd = program.DirectoryPath });
        await dap.Request("configurationDone", expectSuccess: false);
        await dap.Disconnect();
    }
}
