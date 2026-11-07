using System.Text;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class BatchDebuggerTests
{
    [Fact]
    public async Task Large_first_expression_leaves_room_for_all_requested_scalars_within_the_total_value_bound()
    {
        using var program = new Debuggee("batch-value-budget");
        await program.Compile($$"""
            program ValueBudget;
            uses System.Console as Console;
            begin
              const Grid: array of array of int := [{{string.Join(", ", Enumerable.Repeat("new int[64]", 64))}}];
              Console.WriteLine('capture');
            end;
            end program;
            """);
        var requested = new[] { "--break", "Program.fpas:5", "--print", "Grid" }
            .Concat(Enumerable.Range(1, 63).SelectMany(_ => new[] { "--print", "42" })).ToArray();
        using var json = Report(await Run(program, requested), 0);
        var root = json.RootElement;
        var expressions = root.GetProperty("expressions").EnumerateArray().ToArray(); Assert.Equal(64, expressions.Length);
        Assert.All(expressions.Skip(1), item => Assert.Equal("42", item.GetProperty("value").GetProperty("scalar").GetString()));
        static int Count(System.Text.Json.JsonElement value) => 1 + value.GetProperty("children").EnumerateArray().Sum(Count);
        var count = expressions.Sum(item => Count(item.GetProperty("value"))) + root.GetProperty("bindings").EnumerateArray().Sum(item => Count(item.GetProperty("value")));
        Assert.True(count <= 4096, $"Captured {count} values, above the declared bound.");
        Assert.False(root.GetProperty("capture").GetProperty("complete").GetBoolean());
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Explicit_input_and_arguments_are_forwarded_without_reading_command_stdin(bool supplied)
    {
        using var program = new Debuggee("batch-input-λ space");
        await program.Compile("""
            program BatchInput;
            uses System.Console as Console, System.Text.Encoding as Encoding, System.Convert as Convert;
            begin
              Console.OutputEncoding := Encoding.UTF8; Console.InputEncoding := Encoding.UTF8;
              Console.WriteLine(args.Length); Console.WriteLine(Convert.ToBase64String(Encoding.UTF8.GetBytes(args[0])));
              Console.WriteLine(Convert.ToBase64String(Encoding.UTF8.GetBytes(args[1])));
              Console.WriteLine(Console.In.ReadToEnd());
            end;
            end program;
            """);
        File.WriteAllText(program.FilePath("input.bin"), "explicit λ", new UTF8Encoding(false));
        using var json = Report(await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "debug", program.Assembly,
            "--cwd", program.DirectoryPath, .. supplied ? new[] { "--stdin", "input.bin" } : [], "--", "", "λ \"quoted\\"],
            input: "command stdin must be ignored", workingDirectory: Path.GetTempPath()), 0);
        var text = string.Concat(json.RootElement.GetProperty("output").GetProperty("items").EnumerateArray().Select(item => item.GetProperty("text").GetString())).Replace("\r\n", "\n", StringComparison.Ordinal);
        Assert.Equal("2\n\n" + Convert.ToBase64String(Encoding.UTF8.GetBytes("λ \"quoted\\")) + "\n" + (supplied ? "explicit λ" : "") + "\n", text);
    }

    [Fact]
    public async Task Missing_breakpoint_hit_and_unhandled_exception_are_observation_failures()
    {
        using var program = new Debuggee("batch-missed");
        await program.Compile("""
            program Missed;
            uses System.Console as Console;
            function Never(): int;
            begin
              return 7;
            end function;
            begin
              Console.WriteLine('done');
            end;
            end program;
            """);
        using (var json = Report(await Run(program, "--break", "Program.fpas:5"), 3))
        {
            Assert.Equal("completed", json.RootElement.GetProperty("outcome").GetString());
            Assert.Contains(json.RootElement.GetProperty("issues").EnumerateArray(), item => item.GetProperty("code").GetString() == "breakpoint_missed");
        }
        using var failure = new Debuggee("batch-exception");
        await failure.Compile("program Failure; uses System.InvalidOperationException as Failure; begin raise new Failure('batch failure'); end; end program;");
        using var failed = Report(await Run(failure), 3);
        Assert.Equal("exception", failed.RootElement.GetProperty("outcome").GetString());
        Assert.Equal("System.InvalidOperationException", failed.RootElement.GetProperty("exception").GetProperty("type").GetString());
        Assert.Equal("batch failure", failed.RootElement.GetProperty("exception").GetProperty("message").GetProperty("text").GetString());
    }
}
