using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class InteropExecutionTests
{
    [Fact]
    public Task Namespace_imports_constructors_chains_properties_and_structs_execute()
        => ProgramExecution.Output("""
            program Objects;
            uses System as Sys, System.Text as Text, System.Console as Console;
            begin
              const Builder: Text.StringBuilder := new Text.StringBuilder();
              discard Builder.Append('Hello').Append(' CLR');
              Builder.Length := 5;
              Console.WriteLine(Builder.ToString());
              Console.WriteLine(Builder.Length);
              const Date: Sys.DateTime := new Sys.DateTime(2026, 11, 2);
              Console.WriteLine(Date.Year);
              Console.WriteLine(Date.AddDays(1).Day);
              Console.WriteLine(Date.GetType().Name);
              Console.WriteLine(Sys.Math.PI > 3.0);
              const Box: object := 42;
              Console.WriteLine(cast<int>(Box));
            end;
            end program;
            """, "Hello", "5", "2026", "3", "DateTime", "True", "42");

    [Fact]
    public Task Nullable_values_and_reference_flow_execute_without_wrappers()
        => ProgramExecution.Output("""
            program Nulls;
            uses System.Console as Console;
            function Length(Value: string?): int;
            begin if Value = null then return 0; end if; return Value.Length; end function;
            begin
              const Missing: string? := null;
              Console.WriteLine(Length(Missing));
              Console.WriteLine(Missing ?? 'fallback');
              Console.WriteLine(null ?? 3);
              const Number: int? := 42;
              const Empty: int? := null;
              const Wide: double? := Number;
              Console.WriteLine(Number.Value);
              Console.WriteLine(Wide.Value);
              Console.WriteLine(Empty.HasValue);
              Console.WriteLine(Empty ?? 7);
              Console.WriteLine(Missing <> null and Missing.Length > 0);
              Console.WriteLine(Missing = null or Missing.Length = 0);
              var Text: string? := 'known';
              Console.WriteLine(Text.Length);
              Text := null;
              Console.WriteLine(if Text <> null then Text.Length else 9 end if);
            end;
            end program;
            """, "0", "fallback", "3", "42", "42", "False", "7", "False", "True", "5", "9");

    [Fact]
    public Task Using_return_filters_and_finally_execute_against_real_BCL_types()
        => ProgramExecution.Output("""
            program Errors;
            uses System as Sys, System.IO as IO, System.Console as Console;
            function ReadText(): string;
            begin
              using const Reader: IO.StringReader := new IO.StringReader('Read through .NET');
              return Reader.ReadToEnd();
            end function;
            begin
              Console.WriteLine(ReadText());
              try
                raise new Sys.InvalidOperationException('expected');
              catch Failure: Sys.Exception when Failure.Message.Length > 0 do
                Console.WriteLine(Failure.Message);
              finally
                Console.WriteLine('finally');
              end try;
              const Empty: IO.StringReader? := null;
              using const Nothing: IO.StringReader? := Empty;
            end;
            end program;
            """, "Read through .NET", "expected", "finally");
}
