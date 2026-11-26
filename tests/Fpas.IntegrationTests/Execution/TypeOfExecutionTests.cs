// [Documentation](../../../docs/pascal/interop/construction-and-members.md)
using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class TypeOfExecutionTests
{
    [Fact]
    public Task Typeof_uses_CLR_identity_for_primitives_nullable_types_arrays_and_callable_aliases() => ProgramExecution.Output("""
        program TypeIdentity;
        uses System as Sys; uses System.Console as Console; uses System.Collections.Generic as C;
        type Number = int;
        begin
          const Type: Sys.Type := typeof(int);
          Console.WriteLine(Type = typeof(Sys.Int32));
          Console.WriteLine(typeof(Number) = typeof(int));
          Console.WriteLine(typeof(string?) = typeof(string));
          Console.WriteLine(typeof(int?) = typeof(Sys.Nullable of int));
          Console.WriteLine(typeof(int?) <> typeof(int));
          Console.WriteLine(typeof(C.List of string?) = typeof(C.List of string));
          Console.WriteLine(typeof(array of int).IsArray);
          Console.WriteLine(typeof(array[,] of int).GetArrayRank());
          Console.WriteLine(typeof(array of array of int).GetArrayRank());
          Console.WriteLine(typeof(function(int): string) = typeof(Sys.Func of (int, string)));
          Console.WriteLine(typeof(procedure()) = typeof(Sys.Action));
          Console.WriteLine(typeof(Sys.IDisposable).IsInterface);
          Console.WriteLine(typeof(Sys.StringComparison).IsEnum);
          Console.WriteLine(typeof(nint) = typeof(Sys.IntPtr));
          Console.WriteLine(typeof(nuint) = typeof(Sys.UIntPtr));
          const @typeof: int := 7; Console.WriteLine(@typeof);
        end;
        end program;
        """, "True", "True", "True", "True", "True", "True", "True", "2", "1", "True", "True", "True", "True", "True", "True", "7");

    [Fact]
    public Task Generic_type_tokens_keep_type_method_closure_and_async_parameter_scopes() => ProgramExecution.Output("""
        async program GenericTypes;
        uses System as Sys; uses System.Console as Console; uses System.Threading.Tasks as Tasks; uses System.Collections.Generic as C;
        type Box of T = record
          public Value: T;
          public function Get of U(): array of Sys.Type;
          begin return [typeof(T), typeof(U), typeof(Box of T)]; end function;
          public function Reader of U(): function(): array of Sys.Type;
          begin return function(): array of Sys.Type => [typeof(T), typeof(U), typeof(C.KeyValuePair of (T, U))]; end function;
        end record;
        function Get of T(): Sys.Type; begin return typeof(T); end function;
        function NullableValue of T(): Sys.Type; where T: struct; begin return typeof(T?); end function;
        function NullableReference of T(): Sys.Type; where T: class; begin return typeof(T?); end function;
        function Nested of T(): function(): Sys.Type;
          function Local of U(): Sys.Type; begin return typeof(C.KeyValuePair of (T, U)); end function;
        begin
          return Local of string;
        end function;
        async function AsyncGet of T(Gate: Tasks.Task): Tasks.Task of Sys.Type;
        begin await Gate; return typeof(T); end function;
        begin
          Console.WriteLine(Get of int() = typeof(int)); Console.WriteLine(Get of string() = typeof(string));
          Console.WriteLine(NullableValue of int() = typeof(int?));
          Console.WriteLine(NullableReference of string() = typeof(string));
          const Box: Box of int := new Box of int(1);
          const Types: array of Sys.Type := Box.Get of string();
          Console.WriteLine(Types[0] = typeof(int)); Console.WriteLine(Types[1] = typeof(string));
          Console.WriteLine(Types[2] = typeof(Box of int));
          const Closed: array of Sys.Type := Box.Reader of string()();
          Console.WriteLine(Closed[0] = typeof(int)); Console.WriteLine(Closed[1] = typeof(string));
          Console.WriteLine(Closed[2] = typeof(C.KeyValuePair of (int, string)));
          Console.WriteLine(Nested of int()() = typeof(C.KeyValuePair of (int, string)));
          Console.WriteLine((await AsyncGet of int(Tasks.Task.Delay(1))) = typeof(int));
          Console.WriteLine((await AsyncGet of string(Tasks.Task.Delay(1))) = typeof(string));
        end;
        end program;
        """, "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True");
}
