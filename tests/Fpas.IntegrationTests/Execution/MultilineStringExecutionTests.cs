using Fpas.IntegrationTests.Support;

namespace Fpas.IntegrationTests.Execution;

public sealed class MultilineStringExecutionTests
{
    [Theory]
    [InlineData("\n", "\n", "\n")]
    [InlineData("\r\n", "\r\n", "\r\n")]
    [InlineData("\r", "\r", "\r")]
    [InlineData("\r", "\r\n", "\n")]
    public Task Strings_normalize_line_breaks_and_preserve_literal_whitespace(string first, string second, string third)
    {
        var literal = $"'{first}\t A  {second}{third}B''s{first}'";
        return ProgramExecution.Output($$"""
            program Multiline;
            uses System.Console as Console;
            begin
              const Text: string := {{literal}};
              for Character: char in Text do
                Console.Write(cast<int>(Character));
                Console.Write(',');
              end for;
              Console.WriteLine();
            end;
            end program;
            """, "10,9,32,65,32,32,10,10,66,39,115,10,");
    }
}
