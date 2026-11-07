using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class ParameterExecutionTests
{
    [Fact]
    public Task References_alias_fields_vectors_rectangles_and_forwarded_parameters()
        => ProgramExecution.Output("""
            program Parameters;
            uses System as Sys, System.Console as Console;
            procedure Change(ref Value: int; out Previous: int; in Delta: int);
            requires Delta > 0;
            ensures Value = Previous + Delta;
            begin Previous := Value; Value := Value + Delta; end procedure;
            procedure Forward(ref Value: int; out Previous: int; in Delta: int);
            begin Change(ref Value, out Previous, in Delta); end procedure;
            procedure Cleanup(out Value: int);
            ensures Value = 9;
            begin try return; finally Value := 9; end try; end procedure;
            procedure Alias(ref Left: int; ref Right: int);
            begin Left := Left + 1; Right := Right + 2; end procedure;
            procedure Clear(ref Text: string?);
            requires Text <> null;
            ensures Text = null;
            begin Text := null; end procedure;
            begin
              var Value: int := 3; var Previous: int := 0; const Delta: int := 2;
              Forward(ref Value, out Previous, in Delta);
              Console.WriteLine(Value); Console.WriteLine(Previous);
              const Vector: array of int := [1];
              Change(ref Vector[0], out Previous, in Delta); Console.WriteLine(Vector[0]);
              const Grid: array[,] of int := new int[1, 1];
              Change(ref Grid[0, 0], out Previous, in Delta); Console.WriteLine(Grid[0, 0]);
              Cleanup(out Value); Console.WriteLine(Value);
              Console.WriteLine(int.TryParse('42', out Value)); Console.WriteLine(Value);
              Alias(ref Value, ref Value); Console.WriteLine(Value);
              var Optional: string? := 'known'; Clear(ref Optional); Console.WriteLine(Optional ?? 'cleared');
            end;
            end program;
            """, "5", "3", "3", "2", "9", "True", "42", "45", "cleared");

    [Fact]
    public Task Optional_params_named_arguments_and_generic_inference_preserve_written_order()
        => ProgramExecution.Output("""
            program Arguments;
            uses System.Console as Console, System.String as Text;
            function Next(State: array of int): int;
            begin State[0] := State[0] + 1; return State[0]; end function;
            function Pair(Left: int; Right: int := 2 * 3): int;
            begin return Left * 10 + Right; end function;
            function Sum(Seed: int := 3; params Values: array of int): int;
            begin var Total: int := Seed; for Value: int in Values do Total := Total + Value; end for; return Total; end function;
            function Head<T>(params Values: array of T): T;
            begin return Values[0]; end function;
            procedure Copy<T>(in Source: T; out Target: T);
            begin Target := Source; end procedure;
            function Label(Name: string := 'a' + 'b'; Flag: bool := 'x' = 'x'; Optional: int? := null): string;
            begin if Flag and not Optional.HasValue then return Name; else return 'bad'; end if; end function;
            begin
              const State: array of int := [0];
              Console.WriteLine(Pair(Right := Next(State), Left := Next(State)));
              Console.WriteLine(Pair(1));
              Console.WriteLine(Sum()); Console.WriteLine(Sum(1, 2, 3));
              Console.WriteLine(Sum(Values := [4, 5]));
              Console.WriteLine(Head(7, 8)); Console.WriteLine(Head([9, 10]));
              const Source: int := 4; var Target: int := 0;
              Copy(in Source, out Target); Console.WriteLine(Target);
              Console.WriteLine(Text.Format('{0}-{1}', 3, 4));
              Console.WriteLine(Label());
            end;
            end program;
            """, "21", "16", "3", "6", "12", "7", "9", "4", "3-4", "ab");
}
