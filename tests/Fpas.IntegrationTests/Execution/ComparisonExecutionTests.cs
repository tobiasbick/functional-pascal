using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class ComparisonExecutionTests
{
    [Fact]
    public async Task DateTime_TimeSpan_and_Guid_comparisons_use_their_CLR_value_semantics()
        => await ProgramExecution.Output("""
            program Compare;
            uses System as Sys, System.Console as Console;
            var Evaluations: long := 0L;
            function Date(Day: int): Sys.DateTime;
            begin Evaluations := Evaluations * 10 + Day; return new Sys.DateTime(2026, 1, Day); end function;
            begin
              Console.WriteLine(Date(1) = Date(1));
              Console.WriteLine(Evaluations);
              Console.WriteLine(Date(1) <> Date(2));
              Console.WriteLine(Date(1) < Date(2));
              Console.WriteLine(Date(2) > Date(1));
              Console.WriteLine(Date(1) <= Date(1));
              Console.WriteLine(Date(2) >= Date(1));
              const Short: Sys.TimeSpan := Sys.TimeSpan.FromSeconds(1.0);
              const Long: Sys.TimeSpan := Sys.TimeSpan.FromSeconds(2.0);
              Console.WriteLine(Short = Sys.TimeSpan.FromSeconds(1.0));
              Console.WriteLine(Short <> Long);
              Console.WriteLine(Short < Long);
              Console.WriteLine(Long > Short);
              Console.WriteLine(Short <= Short);
              Console.WriteLine(Long >= Short);
              Console.WriteLine(Short > Long);
              Console.WriteLine(new Sys.Guid() = new Sys.Guid());
              Console.WriteLine(new Sys.Guid() <> Sys.Guid.NewGuid());
            end;
            end program;
            """, "True", "11", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "False", "True", "True");
}
