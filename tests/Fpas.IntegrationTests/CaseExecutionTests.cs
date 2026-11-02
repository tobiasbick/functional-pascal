namespace Fpas.IntegrationTests;

public sealed class CaseExecutionTests
{
    [Fact]
    public Task Case_returns_and_loop_exits_run_finally_cleanup() => ProgramExecution.Output("""
        program CaseExits;
        uses System.Console as Console;
        function Read(Value: int): int;
        begin
          try
            case Value of when 0: return 42; else return 7; end case;
          finally Console.WriteLine('return cleanup'); end try;
        end function;
        begin
          for N: int := 0 to 3 do
            try
              case N of when 0: continue; when 1: Console.WriteLine('body'); when 2: break; else Console.WriteLine('wrong'); end case;
            finally Console.WriteLine(N); end try;
          end for;
          Console.WriteLine(Read(0));
        end;
        end program;
        """, "0", "body", "1", "2", "return cleanup", "42");

    [Fact]
    public Task Named_factory_arguments_selector_and_guards_have_single_evaluation() => ProgramExecution.Output("""
        program Evaluation;
        uses System.Console as Console;
        type Pair = union Make(X: int; Y: int); end union;
        function Mark(N: int): int; begin Console.WriteLine(N); return N; end function;
        function Select(): Pair; begin Console.WriteLine('selector'); return Pair.Make(2, 1); end function;
        function Guard(): bool; begin Console.WriteLine('guard'); return false; end function;
        begin
          discard Pair.Make(Y := Mark(1), X := Mark(2));
          case Select() of
            when Pair.Make(1, _), Pair.Make(2, _) if Guard(): Console.WriteLine('wrong');
            when Pair.Make(const X: int, const Y: int): Console.WriteLine(X + Y);
          end case;
        end;
        end program;
        """, "1", "2", "selector", "guard", "3");

    [Fact]
    public Task Null_results_preserve_side_effects_and_nullable_value_bindings_preserve_storage() => ProgramExecution.Output("""
        program NullableCases;
        uses System.Console as Console;
        function Mark(): bool; begin Console.WriteLine('selector'); return true; end function;
        function Size(Text: string?): int;
        begin if Text is string then return Text.Length; else return -1; end if; end function;
        begin
          const Empty: string? := case Mark() of when true: null; when false: null; end case;
          Console.WriteLine(Empty = null);
          const Also: string? := if Mark() then null else null end if;
          Console.WriteLine(Also = null);
          const Number: int? := 42;
          case Number of
            when 42: Console.WriteLine(Number.Value);
            else Console.WriteLine('wrong');
          end case;
          Console.WriteLine(case Number of when is int as N: N; else -1; end case);
          Console.WriteLine(Size('known'));
          Console.WriteLine(case 1 of when int.MinValue..int.MaxValue: 1; else 0; end case);
          Console.WriteLine(case null of when null: 'null'; else 'other'; end case);
          Console.WriteLine(unchecked(case 2147483647 + 1 of when -2147483648: 1; else 0; end case));
        end;
        end program;
        """, "selector", "True", "selector", "True", "42", "42", "5", "1", "null", "1");
}
