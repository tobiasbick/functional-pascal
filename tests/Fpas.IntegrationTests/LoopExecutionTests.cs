namespace Fpas.IntegrationTests;

public sealed class LoopExecutionTests
{
    [Fact]
    public Task Repeat_iteration_cells_and_out_parameter_paths_are_preserved() => ProgramExecution.Output("""
        program RepeatState;
        uses System.Console as Console, System as Sys, System.Collections.Generic as C;
        procedure Parse(out Number: int); begin repeat until int.TryParse('42', out Number); end procedure;
        procedure Initialize(out Number: int);
        begin repeat try continue; finally Number := 7; end try; until Number = 7; end procedure;
        begin
          const Readers: C.List<Sys.Func<int>> := new C.List<Sys.Func<int>>();
          var Index: int := 0;
          repeat
            var Count: int := Index;
            function Next(): int; begin Count := Count + 1; return Count; end function;
            Readers.Add(Next); Index := Index + 1;
          until Index = 3;
          for Read: Sys.Func<int> in Readers do Console.WriteLine(Read()); end for;
          for Read: Sys.Func<int> in Readers do Console.WriteLine(Read()); end for;
          var Number: int := 0; Parse(out Number); Console.WriteLine(Number); Initialize(out Number); Console.WriteLine(Number);
        end;
        end program;
        """, "1", "2", "3", "2", "3", "4", "42", "7");

    [Fact]
    public Task Async_repeat_conditions_and_subrange_steps_survive_suspension() => ProgramExecution.Output("""
        async program AsyncLoops;
        uses System.Console as Console, System.Threading.Tasks as Tasks;
        type High = ulong range 18446744073709551614UL..18446744073709551615UL;
        async function Lower(): Tasks.Task<High>;
        begin await Tasks.Task.Delay(1); return new High(18446744073709551614UL); end function;
        async function Done(Count: int): Tasks.Task<bool>;
        begin await Tasks.Task.Delay(1); return Count = 3; end function;
        begin
          for Index: High := await Lower() to new High(18446744073709551615UL) do
            await Tasks.Task.Delay(1); Console.WriteLine(Index.Value); continue;
          end for;
          var Count: int := 0;
          repeat
            await Tasks.Task.Delay(1); Count := Count + 1;
            if Count = 1 then continue; end if;
          until await Done(Count);
          Console.WriteLine(Count);
        end;
        end program;
        """, "18446744073709551614", "18446744073709551615", "3");

    [Theory]
    [InlineData("sbyte", "-128", "-127", "to")]
    [InlineData("byte", "254", "255", "to")]
    [InlineData("short", "-32767", "-32768", "downto")]
    [InlineData("ushort", "65534", "65535", "to")]
    [InlineData("int", "2147483646", "2147483647", "to")]
    [InlineData("uint", "4294967294U", "4294967295U", "to")]
    [InlineData("long", "-9223372036854775807L", "-9223372036854775808L", "downto")]
    [InlineData("long", "9223372036854775806L", "9223372036854775807L", "to")]
    [InlineData("ulong", "18446744073709551614UL", "18446744073709551615UL", "to")]
    [InlineData("ulong", "1UL", "0UL", "downto")]
    [InlineData("nint", "nint.MinValue + 1", "nint.MinValue", "downto")]
    [InlineData("nuint", "nuint.MaxValue - 1", "nuint.MaxValue", "to")]
    [InlineData("char", "#65534", "#65535", "to")]
    public Task Integral_counters_keep_their_type_and_stop_at_extreme_endpoints(string type, string lower, string upper, string direction)
        => ProgramExecution.Output($$"""
            program Extremes;
            uses System.Console as Console;
            begin
              var Count: int := 0;
              for Index: {{type}} := {{lower}} {{direction}} {{upper}} do
                const Copy: {{type}} := Index; Console.WriteLine(Copy = Index);
                Count := Count + 1; continue;
              end for;
              Console.WriteLine(Count);
            end;
            end program;
            """, "True", "True", "2");

    [Fact]
    public Task Subrange_counters_unwrap_step_and_preserve_per_iteration_captures() => ProgramExecution.Output("""
        program Domains;
        uses System.Console as Console, System as Sys, System.Collections.Generic as C;
        type Port = ushort range 65533..65535;
        type Wide = long range -9223372036854775808L..9223372036854775807L;
        begin
          const Readers: C.List<Sys.Func<Port>> := new C.List<Sys.Func<Port>>();
          for Index: Port := new Port(65534) to new Port(65535) do
            function Read(): Port; begin return Index; end function;
            Readers.Add(Read);
          end for;
          for Read: Sys.Func<Port> in Readers do Console.WriteLine(Read().Value); end for;
          for Index: Wide := new Wide(-9223372036854775807L) downto new Wide(-9223372036854775808L) do Console.WriteLine(Index.Value); end for;
          for Index: Port := new Port(65535) to new Port(65534) do Console.WriteLine('wrong'); end for;
          for Index: ulong := 0UL downto 1UL do Console.WriteLine('wrong'); end for;
        end;
        end program;
        """, "65534", "65535", "-9223372036854775807", "-9223372036854775808");

    [Fact]
    public Task Repeat_runs_once_continue_checks_until_and_break_skips_it() => ProgramExecution.Output("""
        program Repeating;
        uses System.Console as Console;
        var Count: int := 0; var Checks: int := 0;
        function Done(): bool; begin Checks := Checks + 1; return Count = 3; end function;
        function Wrong(): bool; begin Console.WriteLine('wrong'); return true; end function;
        procedure Initialize(out Number: int); begin repeat Number := 42; until true; end procedure;
        function ReturnOnce(): int; begin repeat return 7; until false; end function;
        begin
          repeat
            Count := Count + 1;
            if Count = 2 then continue; end if;
            Console.WriteLine(Count);
          until Done();
          Console.WriteLine(Checks);
          repeat Console.WriteLine('once'); until true;
          repeat break; until Wrong();
          repeat until true;
          var Number: int := 0; Initialize(out Number); Console.WriteLine(Number); Console.WriteLine(ReturnOnce());
        end;
        end program;
        """, "1", "3", "3", "once", "42", "7");

    [Fact]
    public Task Counting_bounds_evaluate_once_before_body_effects() => ProgramExecution.Output("""
        program Bounds;
        uses System.Console as Console;
        var Trace: string := '';
        function Lower(): ulong; begin Trace := Trace + 'L'; return 18446744073709551614UL; end function;
        function Upper(): ulong; begin Trace := Trace + 'U'; return 18446744073709551615UL; end function;
        begin
          for Index: ulong := Lower() to Upper() do Trace := Trace + 'B'; end for;
          Console.WriteLine(Trace);
        end;
        end program;
        """, "LUBB");
}
