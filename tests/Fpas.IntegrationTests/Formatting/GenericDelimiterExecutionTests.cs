// [Documentation](../../../docs/pascal/tools/fmt-style.md)
using Fpas.Compiler.Formatting;
using Fpas.IntegrationTests.Support;

namespace Fpas.IntegrationTests.Formatting;

public sealed class GenericDelimiterExecutionTests
{
    [Fact]
    public async Task Generic_types_next_to_equals_and_arrow_execute_before_and_after_formatting()
    {
        const string source = """
            program Adjacent;
            uses System.Console as Console;
            type Box of T=record public Value:T;end record;
            type Choice of T=variant Empty;Value(Item:T);end variant;
            type IBox of T=protocol Value:T;end protocol;
            begin
              const Make:function():Box of Box of int:=function():Box of Box of int=>new Box of Box of int(new Box of int(42));
              const Result:Box of Box of int:=Make();
              const X:int:=Result.Value.Value;
              if X>=42 then Console.WriteLine(X);end if;
              Console.WriteLine(X>=43);
              Console.WriteLine(X >= 41);
              Console.WriteLine(X>41);
              Console.WriteLine(X<=42);
              Console.WriteLine(X<43);
              Console.WriteLine(X<>43);
              Console.WriteLine(X=42);
            end;end program;
            """;
        var formatted = FpasFormatter.Format(new("adjacent.fpas", source));
        Assert.True(formatted.Success, string.Join('\n', formatted.Diagnostics));
        Assert.Contains("type Box of T = record", formatted.Text);
        Assert.Contains("function(): Box of Box of int =>", formatted.Text);
        foreach (var text in new[] { source, formatted.Text! })
            await ProgramExecution.Output(text, "42", "False", "True", "True", "True", "True", "True", "True");
    }
}
