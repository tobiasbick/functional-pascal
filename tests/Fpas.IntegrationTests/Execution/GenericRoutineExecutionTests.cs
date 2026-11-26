// [Documentation](../../../docs/pascal/language/functions/generic-routines.md)
using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class GenericRoutineExecutionTests
{
    [Fact]
    public Task Generic_routines_delegates_arrays_and_CLR_collections_execute() => ProgramExecution.Output("""
        program Generics;
        uses System.Console as Console; uses System.Collections.Generic as Collections;
        function Identity of T(Value: T): T; begin return Value; end function;
        function Map of (T, U)(Items: array of T; Transform: function(T): U): array of U;
        begin
          const Result: Collections.List of U := new Collections.List of U();
          for Item: T in Items do Result.Add(Transform(Item)); end for;
          return Result.ToArray();
        end function;
        function Repeat of T(Value: T; N: int): T;
        begin if N = 0 then return Value; else return Repeat of T(Value, N - 1); end if; end function;
        begin
          Console.WriteLine(Identity of int(42)); Console.WriteLine(Identity of string('text'));
          const Copy: function(int): int := Identity of int;
          Console.WriteLine(Copy(7)); Console.WriteLine(Repeat of int(3, 4));
          const Values: array of string := Map of (int, string)([1, 2], function(N: int): string => N.ToString());
          for Value: string in Values do Console.WriteLine(Value); end for;
          const Empty: string? := Identity of string?(null);
          Console.WriteLine(Empty = null);
        end;
        end program;
        """, "42", "text", "7", "3", "1", "2", "True");

    [Fact]
    public Task Generic_closures_retain_types_mutable_cells_and_nested_environments() => ProgramExecution.Output("""
        program Closures;
        uses System.Console as Console;
        function Hold of T(Value: T): function(): T;
        begin return function(): T => Value; end function;
        function Switch of T(First: T; Next: T): function(): T;
        begin
          var Current: T := First;
          return function(): T begin const Previous: T := Current; Current := Next; return Previous; end function;
        end function;
        function Nested of T(Value: T): function(): function(): T;
        begin return function(): function(): T => function(): T => Value; end function;
        begin
          Console.WriteLine(Hold of int(42)()); Console.WriteLine(Hold of string('text')());
          const Read: function(): int := Switch of int(1, 2);
          Console.WriteLine(Read()); Console.WriteLine(Read());
          Console.WriteLine(Nested of string('nested')()());
        end;
        end program;
        """, "42", "text", "1", "2", "nested");

    [Fact]
    public Task Constraints_generic_construction_null_validation_and_nullable_values_execute() => ProgramExecution.Output("""
        program Constraints;
        uses System as Sys; uses System.IO as IO; uses System.Console as Console;
        function Compare of T(Left: T; Right: T): int;
        where T: Sys.IComparable of T;
        begin return Left.CompareTo(Right); end function;
        function Create of T(): T; where T: new(); begin return new T(); end function;
        function IsNull of T(Value: T): bool; begin return Value = null; end function;
        function Fallback of T(Value: T; Other: T): T; begin return Value ?? Other; end function;
        function Box of T(Value: T): object?; begin return Value; end function;
        function Describe of T(Value: T): string;
        begin if Value = null then return 'null'; end if; return Value.ToString() ?? 'empty'; end function;
        function Read of T(Value: object): T; begin return cast Value as T; end function;
        function OrDefault of T(Value: T?): T;
        where T: struct;
        begin return Value ?? new T(); end function;
        begin
          Console.WriteLine(Compare of int(1, 2) < 0);
          Console.WriteLine(Compare of string('a', 'b') < 0);
          using const Stream: IO.MemoryStream := Create of IO.MemoryStream();
          Console.WriteLine(Stream.CanRead);
          Console.WriteLine(Describe of string?(null)); Console.WriteLine(Describe of int(42));
          Console.WriteLine(IsNull of int(42)); Console.WriteLine(IsNull of string?(null));
          Console.WriteLine(Fallback of string?(null, 'fallback'));
          Console.WriteLine(Read of int(Box of int(42) ?? 'wrong'));
          Console.WriteLine(OrDefault of int(null)); Console.WriteLine(OrDefault of int(42));
        end;
        end program;
        """, "True", "True", "True", "null", "42", "False", "True", "fallback", "42", "0", "42");
}
