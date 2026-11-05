using Fpas.Compiler.Formatting;
using Fpas.IntegrationTests.Support;

namespace Fpas.IntegrationTests.Formatting;

public sealed class TokenBoundaryExecutionTests
{
    [Fact]
    public async Task Formatting_keeps_word_operators_identifiers_literals_and_arithmetic_distinct()
    {
        const string source = """
            program Boundaries;
            uses System.Console as Console;
            begin
              const Value:bool:=false;
              const notValue:bool:=false;
              Console.WriteLine(notValue);
              Console.WriteLine(not Value);
              Console.WriteLine(not notValue);
              const @event:int:=$F_F;
              Console.WriteLine(@event);
              Console.WriteLine(1--2);
              Console.WriteLine(6.0D/-2.0D);
              Console.WriteLine(6.0D/ // division followed by a comment
                -2.0D);
              Console.WriteLine(6.0D/// a comment followed by subtraction
                -2.0D);
              Console.WriteLine(42.ToString());
              Console.WriteLine('it''s // text');
            end;end program;
            """;
        var formatted = FpasFormatter.Format(new("boundary.fpas", source));
        Assert.True(formatted.Success, string.Join('\n', formatted.Diagnostics));
        foreach (var text in new[] { source, formatted.Text! })
            await ProgramExecution.Output(text, "False", "True", "True", "255", "3", "-3", "-3", "4", "42", "it's // text");
    }
}
