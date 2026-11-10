using Fpas.Compiler.Formatting;
using Fpas.IntegrationTests.Support;

namespace Fpas.IntegrationTests.Formatting;

public sealed class GenericDelimiterExecutionTests
{
    [Fact]
    public async Task Adjacent_generic_closers_equals_and_arrow_compile_and_execute_before_and_after_formatting()
    {
        const string source = """
            program Adjacent;
            uses System.Console as Console;
            type Box<T>=record public Value:T;end record;
            type Choice<T>=variant Empty;Value(Item:T);end variant;
            type IBox<T>=protocol Value:T;end protocol;
            begin
              const Make:function():Box<Box<int>>:=function():Box<Box<int>>=>new Box<Box<int>>(new Box<int>(42));
              const Result:Box<Box<int>>:=Make();
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
        Assert.Contains("type Box<T> = record", formatted.Text);
        Assert.Contains("function(): Box<Box<int>> =>", formatted.Text);
        foreach (var text in new[] { source, formatted.Text! })
            await ProgramExecution.Output(text, "42", "False", "True", "True", "True", "True", "True", "True");
    }
}
