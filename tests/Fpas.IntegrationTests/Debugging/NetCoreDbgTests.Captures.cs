using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Escaping_named_and_lambda_captures_from_a_library_show_live_shared_storage()
    {
        foreach (var named in new[] { true, false })
        {
            using var program = new Debuggee(named ? "named-capture" : "lambda-capture");
            var source = program.FilePath("Library.fpas");
            var body = named ? """
                  function Next(): int;
                  begin
                    discard Label;
                    Count := Count + Step;
                    return Count;
                  end function;
                  return Next;
                """ : """
                  return function(): int
                  begin
                    discard Label;
                    Count := Count + Step;
                    return Count;
                  end function;
                """;
            File.WriteAllText(source, """
                unit CaptureLibrary;
                public function Make<T>(Start: int; Step: int; Label: T): function(): int;
                begin
                  var Count: int := Start;
                """ + "\n" + body + "\nend function;\nend unit;\n");
            var line = Array.FindIndex(File.ReadAllLines(source), text => text.Contains("Count := Count + Step", StringComparison.Ordinal)) + 1;
            await program.CompileFile(source, program.FilePath("Library.dll"), "--target", "library");
            await program.Compile("""
                program Captures;
                uses CaptureLibrary as L, System.Console as Console;
                begin
                  const Counter: function(): int := L.Make<string>(38, 2, 'marker');
                  Console.WriteLine(Counter());
                  Console.WriteLine(Counter());
                end;
                end program;
                """, "--reference", program.FilePath("Library.dll"));
            Assert.Equal("40\n42\n", await program.Run());
            await using var dap = new NetCoreDbgSession(program);
            var entry = await dap.Launch(program);
            await dap.Breakpoints(source, line);
            await dap.Continue(entry.Thread);
            var stop = await dap.Stop();
            Assert.Equal(source, stop.Frame.GetProperty("source").GetProperty("path").GetString());
            Assert.Equal(line, stop.Frame.GetProperty("line").GetInt32());
            await CheckCapture(dap, stop.Frame, "38");
            await dap.Request("next", new { threadId = stop.Thread });
            stop = await dap.Stop("step");
            await CheckCapture(dap, stop.Frame, "40");
            await dap.Continue(stop.Thread);
            stop = await dap.Stop();
            await CheckCapture(dap, stop.Frame, "40");
            await dap.Breakpoints(source);
            Assert.Equal("40\n42\n", await dap.Finish(stop.Thread));
        }
    }

    private static async Task CheckCapture(NetCoreDbgSession dap, JsonElement frame, string expected)
    {
        var receiver = Value(await dap.Locals(frame), "this");
        var fields = await dap.Variables(receiver.GetProperty("variablesReference").GetInt32());
        Value(fields, "Step", "2");
        Value(fields, "Label", "\"marker\"");
        var cell = Value(fields, "Count");
        Value(await dap.Variables(cell.GetProperty("variablesReference").GetInt32()), "Value", expected);
    }

    [NetCoreDbgFact]
    public async Task Nested_capture_names_distinguish_shadowed_source_bindings()
    {
        using var program = new Debuggee("shadowed-captures");
        await program.Compile("""
            program Shadowed;
            uses System.Console as Console;
            function Make(Value: int): function(): int;
            begin
              function ReadOuter(): int;
              begin return Value; end function;
              begin
                const Value: int := 2;
                function ReadInner(): int;
                begin
                  return Value + ReadOuter();
                end function;
                return ReadInner;
              end;
            end function;
            begin
              const Read: function(): int := Make(40);
              Console.WriteLine(Read());
            end;
            end program;
            """);
        Assert.Equal("42\n", await program.Run());
        await using var dap = new NetCoreDbgSession(program);
        var entry = await dap.Launch(program, 11);
        await dap.Continue(entry.Thread);
        var stop = await dap.Stop();
        var receiver = Value(await dap.Locals(stop.Frame), "this");
        var fields = await dap.Variables(receiver.GetProperty("variablesReference").GetInt32());
        Value(fields, "Value", "2"); Value(fields, "Value_1", "40");
        Assert.Equal("42\n", await dap.Finish(stop.Thread));
    }
}
