namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Source_aggregates_domains_nullables_arrays_and_generics_expose_their_values()
    {
        using var program = new Debuggee("values");
        await program.Compile("""
            program Values;
            uses System.Console as Console;
            type Point = record X: int; Y: int; end record;
            type UserId = distinct long;
            type Port = ushort range 100..200;
            type Box<T> = record Item: T; end record;
            type Outcome<T> = union Success(Value: T); Missing; end union;
            begin
              const Position: Point := new Point(20, 22);
              const Id: UserId := new UserId(42L);
              const Selected: Port := new Port(142);
              const Ports: Port[] := new Port[1];
              const Wrapped: Box<UserId> := new Box<UserId>(Id);
              const Result: Outcome<string> := Outcome.Success('Grüße');
              const Missing: Outcome<string> := Outcome.Missing;
              const Present: int? := 42;
              const Absent: int? := null;
              const Numbers: int[] := [20, 22];
              Console.WriteLine(Position.X + Position.Y);
              Console.WriteLine(Id.Value);
              Console.WriteLine(Selected.Value);
              Console.WriteLine(Ports[0].Value);
              Console.WriteLine(Wrapped.Item.Value);
              Console.WriteLine(case Result of when Outcome.Success(const Text: string): Text; else ''; end case);
              Console.WriteLine(Missing.Tag);
              Console.WriteLine(Present ?? 0);
              Console.WriteLine(Absent ?? -1);
              Console.WriteLine(Numbers[0] + Numbers[1]);
            end;
            end program;
            """);
        const string expected = "42\n42\n142\n100\n42\nGrüße\n1\n42\n-1\n42\n";
        Assert.Equal(expected, await program.Run());
        await using var dap = new NetCoreDbgSession(program);
        var line = Array.FindIndex(File.ReadAllLines(program.Source), text => text.Contains("Console.WriteLine(Position.X", StringComparison.Ordinal)) + 1;
        var entry = await dap.Launch(program, line);
        await dap.Continue(entry.Thread);
        var stop = await dap.Stop();
        Assert.Equal(line, stop.Frame.GetProperty("line").GetInt32());
        var locals = await dap.Locals(stop.Frame);
        var point = await dap.Variables(Value(locals, "Position").GetProperty("variablesReference").GetInt32());
        var id = await dap.Variables(Value(locals, "Id").GetProperty("variablesReference").GetInt32());
        var port = await dap.Variables(Value(locals, "Selected").GetProperty("variablesReference").GetInt32());
        var ports = await dap.Variables(Value(locals, "Ports").GetProperty("variablesReference").GetInt32());
        var defaultPort = await dap.Variables(Value(ports, "[0]").GetProperty("variablesReference").GetInt32());
        var box = await dap.Variables(Value(locals, "Wrapped").GetProperty("variablesReference").GetInt32());
        var boxedId = await dap.Variables(Value(box, "Item").GetProperty("variablesReference").GetInt32());
        var result = await dap.Variables(Value(locals, "Result").GetProperty("variablesReference").GetInt32());
        var missing = await dap.Variables(Value(locals, "Missing").GetProperty("variablesReference").GetInt32());
        var numbers = await dap.Variables(Value(locals, "Numbers").GetProperty("variablesReference").GetInt32());
        Value(point, "X", "20"); Value(point, "Y", "22");
        Value(id, "Value", "42"); Value(port, "Value", "142"); Value(defaultPort, "Value", "100");
        Value(boxedId, "Value", "42");
        Value(result, "Value", "\"Grüße\""); Value(result, "Tag", "0"); Value(missing, "Tag", "1");
        Value(locals, "Present", "42"); Value(locals, "Absent", "null");
        Value(numbers, "[0]", "20"); Value(numbers, "[1]", "22");
        Assert.DoesNotContain(point.Concat(box).Concat(result), field => field.GetProperty("name").GetString()!.Contains("BackingField", StringComparison.Ordinal));
        Assert.Equal(expected, await dap.Finish(stop.Thread));
    }
}
