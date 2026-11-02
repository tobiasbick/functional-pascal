namespace Fpas.IntegrationTests;

public sealed class CallableExecutionTests
{
    [Fact]
    public Task Named_routines_callable_parameters_returns_and_both_lambda_bodies_execute()
        => ProgramExecution.Output("""
            program Functions;
            uses System as Sys, System.Console as Console;
            function Double(Value: int): int;
            begin return Value * 2; end function;
            function Apply(Transform: function(int): int; Value: int): int;
            begin return Transform(Value); end function;
            function Adder(Amount: int): function(int): int;
            begin return function(Value: int): int => Value + Amount; end function;
            begin
              const Transform: function(int): int := Double;
              Console.WriteLine(Apply(Transform, 3));
              Console.WriteLine(Apply(function(Value: int): int begin return Double(Value) + 1; end function, 4));
              Console.WriteLine(Adder(10)(2));
              const Print: procedure(string) := procedure(Text: string) begin Console.WriteLine(Text); end procedure;
              Print('delegate');
              const ReadArgs: function(): int := function(): int => args.Length;
              Console.WriteLine(ReadArgs());
              const Nullable: Sys.Func<int, int>? := Transform;
              if Nullable <> null then Console.WriteLine(Nullable(7)); end if;
            end;
            end program;
            """, "6", "9", "12", "delegate", "0", "14");

    [Fact]
    public Task Mutable_cells_are_shared_and_nested_closures_outlive_their_calls()
        => ProgramExecution.Output("""
            program Captures;
            uses System.Console as Console;
            function Factory(Start: int): function(): function(): int;
            begin
              var Value: int := Start;
              return function(): function(): int => function(): int begin Value := Value + 1; return Value; end function;
            end function;
            begin
              var Value: int := 1;
              const Increment: function(): int := function(): int begin Value := Value + 1; return Value; end function;
              const Read: function(): int := function(): int => Value;
              Console.WriteLine(Increment());
              Console.WriteLine(Read());
              Value := 9;
              Console.WriteLine(Read());
              const Make: function(): function(): int := Factory(20);
              const First: function(): int := Make();
              const Second: function(): int := Make();
              Console.WriteLine(First()); Console.WriteLine(Second()); Console.WriteLine(First());
              const Separate: function(): int := Factory(30)();
              Console.WriteLine(Separate());
            end;
            end program;
            """, "2", "2", "9", "21", "22", "23", "31");

    [Fact]
    public Task Loop_captures_keep_iteration_values_and_fresh_local_cells()
        => ProgramExecution.Output("""
            program Iterations;
            uses System as Sys, System.Collections.Generic as C, System.Console as Console;
            begin
              const Functions: C.List<Sys.Func<int>> := new C.List<Sys.Func<int>>();
              for Index: int := 1 to 3 do
                const Copy: int := Index;
                var Mutable: int := Index;
                Functions.Add(function(): int => Index + Copy + Mutable);
                Mutable := Mutable + 10;
              end for;
              for Read: Sys.Func<int> in Functions do Console.WriteLine(Read()); end for;
              const More: C.List<Sys.Func<int>> := new C.List<Sys.Func<int>>();
              for Value: int in [4, 5] do More.Add(function(): int => Value); end for;
              for Read: Sys.Func<int> in More do Console.WriteLine(Read()); end for;
            end;
            end program;
            """, "13", "16", "19", "4", "5");

    [Fact]
    public Task Explicit_generic_Linq_calls_bind_typed_callbacks_and_preserve_laziness()
        => ProgramExecution.Output("""
            program Pipeline;
            uses System.Linq.Enumerable as Seq, System.Collections.Generic as C, System.Console as Console;
            function Label(Value: int): string;
            begin return 'item:' + Value.ToString(); end function;
            begin
              var Calls: int := 0;
              const Threshold: int := 2;
              const Filtered: C.IEnumerable<int> := Seq.Where<int>([1, 2, 3, 4],
                function(Value: int): bool begin Calls := Calls + 1; return Value > Threshold; end function);
              const Labels: C.IEnumerable<string> := Seq.Select<int, string>(Filtered, Label);
              Console.WriteLine(Calls);
              for Text: string in Labels do Console.WriteLine(Text); end for;
              Console.WriteLine(Calls);
              const Array: string[] := Seq.ToArray<string>(Labels);
              Console.WriteLine(Array.Length);
              Console.WriteLine(Calls);
              const Values: C.List<int> := new C.List<int>([3, 1, 2]);
              Values.Sort(function(Left: int; Right: int): int => Right - Left);
              Console.WriteLine(Values[0]);
              Console.WriteLine(Values.Find(function(Value: int): bool => Value = 2));
            end;
            end program;
            """, "0", "item:3", "item:4", "4", "2", "8", "3", "2");

    [Fact]
    public Task Delegate_exceptions_and_lambdas_declared_in_finally_use_their_own_control_flow()
        => ProgramExecution.Output("""
            program Cleanup;
            uses System as Sys, System.IO as IO, System.Console as Console;
            begin
              const Fail: procedure() := procedure() begin
                using const Reader: IO.StringReader := new IO.StringReader('text');
                raise new Sys.InvalidOperationException(Reader.ReadToEnd());
              end procedure;
              try Fail();
              catch Failure: Sys.InvalidOperationException do Console.WriteLine(Failure.Message);
              finally
                const Read: function(): int := function(): int begin return 9; end function;
                Console.WriteLine(Read());
              end try;
              var Text: string? := 'known';
              const Length: function(): int := function(): int begin
                if Text = null then return 0; end if;
                return Text.Length;
              end function;
              Console.WriteLine(Length()); Text := null; Console.WriteLine(Length());
            end;
            end program;
            """, "text", "9", "5", "0");

    [Fact]
    public Task Lambda_arithmetic_uses_its_lexical_checked_context()
        => ProgramExecution.Output("""
            program Arithmetic;
            uses System as Sys, System.Console as Console;
            begin
              const Wrap: function(int): int := unchecked(function(Value: int): int => Value + 1);
              const Check: function(int): int := function(Value: int): int => Value + 1;
              Console.WriteLine(Wrap(2147483647));
              try discard Check(2147483647);
              catch Failure: Sys.OverflowException do Console.WriteLine('checked'); end try;
            end;
            end program;
            """, "-2147483648", "checked");
}
