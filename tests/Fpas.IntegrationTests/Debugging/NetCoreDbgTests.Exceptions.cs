namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Handled_exception_continues_and_user_unhandled_exception_stops_in_source()
    {
        using var program = new Debuggee("exceptions");
        await program.Compile("""
            program Fail;
            uses System as Sys, System.Console as Console;
            procedure Work();
            begin
              try
                raise new Sys.ArgumentException('handled');
              catch Failure: Sys.ArgumentException do
                Console.WriteLine('handled');
              end try;
              raise new Sys.InvalidOperationException('uncaught');
            end procedure;
            begin Work(); end;
            end program;
            """);
        await using var dap = new NetCoreDbgSession(program);
        var entry = await dap.Launch(program);
        await dap.Continue(entry.Thread);
        var stop = await dap.Stop("exception");
        Assert.Equal(10, stop.Frame.GetProperty("line").GetInt32());
        var exception = await dap.Request("exceptionInfo", new { threadId = stop.Thread });
        Assert.Contains("InvalidOperationException", exception.GetProperty("exceptionId").GetString());
        await dap.Disconnect();
    }
}
