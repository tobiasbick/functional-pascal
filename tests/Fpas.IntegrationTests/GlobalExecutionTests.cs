namespace Fpas.IntegrationTests;

public sealed class GlobalExecutionTests
{
    [Fact]
    public Task Global_initializers_run_in_written_order_and_closures_read_shared_storage() => ProgramExecution.Output("""
        program Globals;
        uses System.Console as Console;
        var Trace: string := '';
        const First: int := Mark(1);
        function Mark(Value: int): int;
        begin Trace := Trace + Value.ToString(); return Value; end function;
        var Second: int := Mark(2);
        var Counter: long := 3L;
        const Read: function(): long := function(): long => Counter;
        const Saved: function(): long := Capture();
        const Items: int[] := [10];
        function Capture(): function(): long;
        begin const Local: long := 40L; return function(): long => Local + Counter; end function;
        procedure Increment(ref Value: long); begin Value := Value + 1L; end procedure;
        procedure Set(out Value: long); begin Value := 8L; end procedure;
        function Inspect(in Value: long): long; begin return Value; end function;
        function Shadow(Counter: long): long; begin return Counter; end function;
        begin
          Console.WriteLine(Trace); Console.WriteLine(First + Second);
          Console.WriteLine(Read()); Increment(ref Counter); Console.WriteLine(Read());
          Console.WriteLine(Saved()); Set(out Counter); Console.WriteLine(Inspect(in Counter));
          Console.WriteLine(Shadow(100L)); Console.WriteLine(Counter);
          Items[0] := 12; Console.WriteLine(Items[0]);
          begin const Counter: long := 50L; Console.WriteLine(Counter); end;
          Console.WriteLine(Counter);
        end;
        end program;
        """, "12", "3", "3", "4", "44", "8", "100", "8", "12", "50", "8");

    [Fact]
    public Task Async_global_delegates_share_fields_after_suspension() => ProgramExecution.Output("""
        async program GlobalAsync;
        uses System.Console as Console, System.Threading.Tasks as Tasks;
        var Counter: int := 3;
        const Gate: Tasks.TaskCompletionSource<bool> := new Tasks.TaskCompletionSource<bool>();
        const Read: function(): Tasks.Task<int> := async function(): Tasks.Task<int>
        begin discard await Gate.Task; return Counter; end function;
        begin
          const Pending: Tasks.Task<int> := Read();
          Counter := 9; Gate.SetResult(true);
          Console.WriteLine(await Pending); Console.WriteLine(Counter);
        end;
        end program;
        """, "9", "9");

    [Fact]
    public Task Forward_field_reads_keep_CLR_default_storage_semantics() => ProgramExecution.Output("""
        program Forward;
        uses System.Console as Console;
        const Earlier: int := Later;
        var Later: int := 7;
        begin Console.WriteLine(Earlier); Console.WriteLine(Later); end;
        end program;
        """, "0", "7");

    [Fact]
    public Task Record_methods_and_lexical_bindings_take_precedence_over_outer_globals() => ProgramExecution.Output("""
        program Scope;
        uses System.Console as Console;
        var Count: int := 3;
        type Reader = record
          function Count(): int; begin return 42; end function;
          function Read(): int; begin return Count(); end function;
          function Saved(): function(): int; begin return function(): int => Count(); end function;
        end record;
        begin
          const Value: Reader := new Reader();
          Console.WriteLine(Value.Read()); Console.WriteLine(Value.Saved()()); Console.WriteLine(Count);
        end;
        end program;
        """, "42", "42", "3");

    [Fact]
    public async Task A_failed_program_initializer_prevents_entry_and_preserves_the_CLR_exception()
    {
        var result = await ProgramExecution.Run("""
            program Failure;
            uses System.Console as Console;
            const Value: int := int.Parse('invalid');
            begin Console.WriteLine('entry'); end;
            end program;
            """);
        Assert.NotEqual(0, result.ExitCode); Assert.Equal("", result.Output);
        Assert.Contains("TypeInitializationException", result.Error); Assert.Contains("FormatException", result.Error);
    }
}
