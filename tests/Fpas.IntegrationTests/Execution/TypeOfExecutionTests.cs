using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class TypeOfExecutionTests
{
    [Fact]
    public Task Typeof_uses_CLR_identity_for_primitives_nullable_types_arrays_and_callable_aliases() => ProgramExecution.Output("""
        program TypeIdentity;
        uses System as Sys, System.Console as Console, System.Collections.Generic as C;
        type Number = int;
        begin
          const Type: Sys.Type := typeof(int);
          Console.WriteLine(Type = typeof(Sys.Int32));
          Console.WriteLine(typeof(Number) = typeof(int));
          Console.WriteLine(typeof(string?) = typeof(string));
          Console.WriteLine(typeof(int?) = typeof(Sys.Nullable<int>));
          Console.WriteLine(typeof(int?) <> typeof(int));
          Console.WriteLine(typeof(C.List<string?>) = typeof(C.List<string>));
          Console.WriteLine(typeof(int[]).IsArray);
          Console.WriteLine(typeof(int[,]).GetArrayRank());
          Console.WriteLine(typeof(int[][]).GetArrayRank());
          Console.WriteLine(typeof(function(int): string) = typeof(Sys.Func<int, string>));
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
        uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks, System.Collections.Generic as C;
        type Box<T> = record
          Value: T;
          function Get<U>(): Sys.Type[];
          begin return [typeof(T), typeof(U), typeof(Box<T>)]; end function;
          function Reader<U>(): function(): Sys.Type[];
          begin return function(): Sys.Type[] => [typeof(T), typeof(U), typeof(C.KeyValuePair<T, U>)]; end function;
        end record;
        function Get<T>(): Sys.Type; begin return typeof(T); end function;
        function NullableValue<T>(): Sys.Type; where T: struct; begin return typeof(T?); end function;
        function NullableReference<T>(): Sys.Type; where T: class; begin return typeof(T?); end function;
        function Nested<T>(): function(): Sys.Type;
        begin
          function Local<U>(): Sys.Type; begin return typeof(C.KeyValuePair<T, U>); end function;
          return Local<string>;
        end function;
        async function AsyncGet<T>(Gate: Tasks.Task): Tasks.Task<Sys.Type>;
        begin await Gate; return typeof(T); end function;
        begin
          Console.WriteLine(Get<int>() = typeof(int)); Console.WriteLine(Get<string>() = typeof(string));
          Console.WriteLine(NullableValue<int>() = typeof(int?));
          Console.WriteLine(NullableReference<string>() = typeof(string));
          const Box: Box<int> := new Box<int>(1);
          const Types: Sys.Type[] := Box.Get<string>();
          Console.WriteLine(Types[0] = typeof(int)); Console.WriteLine(Types[1] = typeof(string));
          Console.WriteLine(Types[2] = typeof(Box<int>));
          const Closed: Sys.Type[] := Box.Reader<string>()();
          Console.WriteLine(Closed[0] = typeof(int)); Console.WriteLine(Closed[1] = typeof(string));
          Console.WriteLine(Closed[2] = typeof(C.KeyValuePair<int, string>));
          Console.WriteLine(Nested<int>()() = typeof(C.KeyValuePair<int, string>));
          Console.WriteLine((await AsyncGet<int>(Tasks.Task.Delay(1))) = typeof(int));
          Console.WriteLine((await AsyncGet<string>(Tasks.Task.Delay(1))) = typeof(string));
        end;
        end program;
        """, "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True", "True");
}
