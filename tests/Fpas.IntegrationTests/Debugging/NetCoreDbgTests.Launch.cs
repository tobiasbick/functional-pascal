using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Pending_breakpoint_binds_without_entry_stop_with_relative_CLI_source_args_cwd_and_output()
    {
        using var program = new Debuggee("launch");
        File.WriteAllText(program.Source, """
            program Launch;
            uses System as Sys, System.IO as IO, System.Console as Console;
            function Double(Input: int): int;
            begin
              return Input * 2;
            end function;
            begin
              Console.WriteLine(Sys.Environment.GetCommandLineArgs()[1]);
              Console.Error.WriteLine('stderr marker');
              Console.WriteLine(IO.File.ReadAllText('input.txt'));
              const Answer: int := Double(21);
              Console.WriteLine(Answer);
            end;
            end program;
            """);
        var built = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "Program.fpas",
            "--output", program.Assembly, "--reference-dir", TestEnvironment.ReferenceDirectory], workingDirectory: program.DirectoryPath);
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var cwd = program.FilePath("work"); Directory.CreateDirectory(cwd);
        File.WriteAllText(Path.Combine(cwd, "input.txt"), "working directory marker");
        await using var dap = new NetCoreDbgSession(program);
        var pending = await dap.Start(program, [11], ["Grüße from DAP"], cwd);
        Assert.False(pending.GetProperty("breakpoints")[0].GetProperty("verified").GetBoolean());
        var stop = await dap.Stop();
        Assert.Equal(11, stop.Frame.GetProperty("line").GetInt32());
        Assert.Equal(program.Source, stop.Frame.GetProperty("source").GetProperty("path").GetString());
        var bound = await dap.Event("breakpoint");
        Assert.True(bound.GetProperty("breakpoint").GetProperty("verified").GetBoolean());
        await dap.Request("stepIn", new { threadId = stop.Thread });
        stop = await dap.Stop("step");
        Assert.Equal(5, stop.Frame.GetProperty("line").GetInt32());
        Value(await dap.Locals(stop.Frame), "Input", "21");
        await dap.Request("stepOut", new { threadId = stop.Thread });
        stop = await dap.Stop("step");
        Assert.Equal(program.Source, stop.Frame.GetProperty("source").GetProperty("path").GetString());
        Assert.Contains(stop.Frame.GetProperty("line").GetInt32(), new[] { 11, 12 });
        Assert.Equal("Grüße from DAP\nworking directory marker\n42\n", await dap.Finish(stop.Thread));
        Assert.Equal("stderr marker\n", dap.Output("stderr"));
    }
}
