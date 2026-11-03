using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class CollectionsExecutionTests
{
    [Fact]
    public async Task Entry_arguments_preserve_spaces_unicode_and_exclude_the_executable()
    {
        var result = await ProgramExecution.Run("""
            program Arguments;
            uses System.Console as Console;
            begin
              Console.WriteLine(args.Length);
              for Arg: string in args do Console.WriteLine(Arg); end for;
            end;
            end program;
            """, "with spaces", "Grüße", "");
        Assert.Equal(0, result.ExitCode);
        Assert.Equal("3\nwith spaces\nGrüße\n\n".Replace("\n", Environment.NewLine, StringComparison.Ordinal), result.Output);
    }

    [Fact]
    public Task Vectors_jagged_arrays_rectangles_and_utf16_characters_execute()
        => ProgramExecution.Output("""
            program Arrays;
            uses System as Sys, System.Console as Console;
            begin
              const Values: int[] := [1, 2, 3];
              const Alias: int[] := Values;
              Alias[1] := 9;
              Console.WriteLine(Values[1]);
              const Empty: string[] := [];
              Console.WriteLine(Empty.Length);
              const Jagged: int[][] := [[], [4, 5]];
              Jagged[1][0] := 7;
              Console.WriteLine(Jagged[1][0]);
              const Grid: int[,] := new int[2, 3];
              Grid[1, 2] := 8;
              Console.WriteLine(Grid.Length);
              for Cell: int in Grid do Console.WriteLine(Cell); end for;
              const Optional: string?[] := new string?[1];
              Console.WriteLine(Optional[0] ?? 'null element');
              const Zero: string[,] := new string[0, 3];
              Console.WriteLine(Zero.Length);
              const Text: string := 'A😀';
              Console.WriteLine(Text.Length);
              Console.WriteLine(Text[0]);
              for Unit: char in Text do Console.WriteLine(cast<int>(Unit)); end for;
              Console.WriteLine('line' + #10 + 'two');
              Console.WriteLine(#65 + 1);
              try discard Values[3]; catch Failure: Sys.IndexOutOfRangeException do Console.WriteLine('bounds'); end try;
            end;
            end program;
            """, "9", "0", "7", "6", "0", "0", "0", "0", "0", "8", "null element", "0", "3", "A", "65", "55357", "56832", "line\ntwo", "66", "bounds");

    [Fact]
    public Task Generic_methods_fields_indexers_nested_types_and_interface_variance_execute()
        => ProgramExecution.Output("""
            program Collections;
            uses System.Collections.Generic as C, System.Console as Console;
            begin
              const Values: C.List<string> := new C.List<string>();
              Values.Add('first'); Values.Add('second');
              Values[0] := 'updated';
              Console.WriteLine(Values[0]);
              Console.WriteLine(Values.Count);
              const Indexed: C.IList<string> := Values;
              Indexed.Add('third'); Indexed[2] := 'third';
              Console.WriteLine(Indexed.Count);
              const Readable: C.IEnumerable<object> := Values;
              for Value: object in Readable do Console.WriteLine(Value); end for;
              const Map: C.Dictionary<string, int> := new C.Dictionary<string, int>();
              Map['one'] := 1; Map['two'] := 2;
              Console.WriteLine(Map['two']);
              for Value: int in Map.Values do Console.WriteLine(Value); end for;
              for Pair: C.KeyValuePair<string, int> in Map do Console.WriteLine(Pair.Key); Console.WriteLine(Pair.Value); end for;
              const Nullable: C.List<string?> := new C.List<string?>();
              Nullable.Add(null); Nullable.Add('present');
              for Value: string? in Nullable do Console.WriteLine(Value ?? 'absent'); end for;
              const Nested: C.List<C.List<int>> := new C.List<C.List<int>>();
              Nested.Add(new C.List<int>([3, 4]));
              Console.WriteLine(Nested[0][1]);
              Console.WriteLine(C.Comparer<int>.Default.Compare(1, 2) < 0);
            end;
            end program;
            """, "updated", "2", "3", "updated", "second", "third", "2", "1", "2", "one", "1", "two", "2", "absent", "present", "4", "True");

    [Fact]
    public Task Empty_literals_receive_source_parameter_return_cast_and_conditional_context()
        => ProgramExecution.Output("""
            program Context;
            uses System.Console as Console;
            function Echo(Values: int[]): int[];
            begin return Values; end function;
            function Empty(Present: bool): int[];
            begin return if Present then [] else [1] end if; end function;
            begin
              Console.WriteLine(Echo([]).Length);
              Console.WriteLine(Empty(true).Length);
              Console.WriteLine(cast<int[]>([]).Length);
            end;
            end program;
            """, "0", "0", "0");

    [Fact]
    public Task Enumeration_preserves_collection_invalidation_and_loop_control()
        => ProgramExecution.Output("""
            program Loops;
            uses System as Sys, System.Collections.Generic as C, System.Console as Console;
            function Source(): int[];
            begin Console.WriteLine('source'); return [1, 2, 3]; end function;
            begin
              for Item: int in Source() do
                if Item = 1 then continue; end if;
                Console.WriteLine(Item);
                break;
              end for;
              const Values: C.List<int> := new C.List<int>([1, 2]);
              try
                for Item: int in Values do Values.Add(3); end for;
              catch Failure: Sys.InvalidOperationException do Console.WriteLine('invalidated'); end try;
              for Outer: int in [1, 2] do
                for Inner: int in [3, 4] do Console.WriteLine(Outer * 10 + Inner); break; end for;
              end for;
            end;
            end program;
            """, "source", "2", "invalidated", "13", "23");
}
