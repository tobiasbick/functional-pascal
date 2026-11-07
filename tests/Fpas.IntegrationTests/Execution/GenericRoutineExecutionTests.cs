using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class GenericRoutineExecutionTests
{
    [Fact]
    public Task Generic_routines_delegates_arrays_and_CLR_collections_execute() => ProgramExecution.Output("""
        program Generics;
        uses System.Console as Console, System.Collections.Generic as Collections;
        function Identity<T>(Value: T): T; begin return Value; end function;
        function Map<T, U>(Items: array of T; Transform: function(T): U): array of U;
        begin
          const Result: Collections.List<U> := new Collections.List<U>();
          for Item: T in Items do Result.Add(Transform(Item)); end for;
          return Result.ToArray();
        end function;
        function Repeat<T>(Value: T; N: int): T;
        begin if N = 0 then return Value; else return Repeat<T>(Value, N - 1); end if; end function;
        begin
          Console.WriteLine(Identity<int>(42)); Console.WriteLine(Identity<string>('text'));
          const Copy: function(int): int := Identity<int>;
          Console.WriteLine(Copy(7)); Console.WriteLine(Repeat<int>(3, 4));
          const Values: array of string := Map<int, string>([1, 2], function(N: int): string => N.ToString());
          for Value: string in Values do Console.WriteLine(Value); end for;
          const Empty: string? := Identity<string?>(null);
          Console.WriteLine(Empty = null);
        end;
        end program;
        """, "42", "text", "7", "3", "1", "2", "True");

    [Fact]
    public Task Generic_closures_retain_types_mutable_cells_and_nested_environments() => ProgramExecution.Output("""
        program Closures;
        uses System.Console as Console;
        function Hold<T>(Value: T): function(): T;
        begin return function(): T => Value; end function;
        function Switch<T>(First: T; Next: T): function(): T;
        begin
          var Current: T := First;
          return function(): T begin const Previous: T := Current; Current := Next; return Previous; end function;
        end function;
        function Nested<T>(Value: T): function(): function(): T;
        begin return function(): function(): T => function(): T => Value; end function;
        begin
          Console.WriteLine(Hold<int>(42)()); Console.WriteLine(Hold<string>('text')());
          const Read: function(): int := Switch<int>(1, 2);
          Console.WriteLine(Read()); Console.WriteLine(Read());
          Console.WriteLine(Nested<string>('nested')()());
        end;
        end program;
        """, "42", "text", "1", "2", "nested");

    [Fact]
    public Task Constraints_generic_construction_null_validation_and_nullable_values_execute() => ProgramExecution.Output("""
        program Constraints;
        uses System as Sys, System.IO as IO, System.Console as Console;
        function Compare<T>(Left: T; Right: T): int;
        where T: Sys.IComparable<T>;
        begin return Left.CompareTo(Right); end function;
        function Create<T>(): T; where T: new(); begin return new T(); end function;
        function IsNull<T>(Value: T): bool; begin return Value = null; end function;
        function Fallback<T>(Value: T; Other: T): T; begin return Value ?? Other; end function;
        function Box<T>(Value: T): object?; begin return Value; end function;
        function Describe<T>(Value: T): string;
        begin if Value = null then return 'null'; end if; return Value.ToString() ?? 'empty'; end function;
        function Read<T>(Value: object): T; begin return cast<T>(Value); end function;
        function OrDefault<T>(Value: T?): T;
        where T: struct;
        begin return Value ?? new T(); end function;
        begin
          Console.WriteLine(Compare<int>(1, 2) < 0);
          Console.WriteLine(Compare<string>('a', 'b') < 0);
          using const Stream: IO.MemoryStream := Create<IO.MemoryStream>();
          Console.WriteLine(Stream.CanRead);
          Console.WriteLine(Describe<string?>(null)); Console.WriteLine(Describe<int>(42));
          Console.WriteLine(IsNull<int>(42)); Console.WriteLine(IsNull<string?>(null));
          Console.WriteLine(Fallback<string?>(null, 'fallback'));
          Console.WriteLine(Read<int>(Box<int>(42) ?? 'wrong'));
          Console.WriteLine(OrDefault<int>(null)); Console.WriteLine(OrDefault<int>(42));
        end;
        end program;
        """, "True", "True", "True", "null", "42", "False", "True", "fallback", "42", "0", "42");
}
