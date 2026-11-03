using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class OverloadExecutionTests
{
    [Fact]
    public Task Source_overloads_resolve_generics_delegates_defaults_modes_and_record_methods()
        => ProgramExecution.Output("""
            program Overloads;
            uses System.Console as Console;
            function Choose(Value: int): string; begin return 'int'; end function;
            function Choose(Value: double): string; begin return 'double'; end function;
            function Choose<T>(Value: T): string; begin return 'generic'; end function;
            function Choose(): string; begin return 'empty'; end function;
            function Mode(Value: int): string; begin return 'value'; end function;
            function Mode(in Value: int): string; begin return 'in'; end function;
            function Many(Value: int): string; begin return 'fixed'; end function;
            function Many(params Values: int[]): string; begin return 'params'; end function;
            type Box = record
              Value: int;
              function Read(): int; begin return self.Value; end function;
              function Read(Delta: int): int; begin return self.Read() + Delta; end function;
            end record;
            begin
              Console.WriteLine(Choose(1)); Console.WriteLine(Choose(1.5)); Console.WriteLine(Choose('text'));
              Console.WriteLine(Choose<int>(2)); Console.WriteLine(Choose());
              const Callback: function(int): string := Choose;
              Console.WriteLine(Callback(3));
              const Value: int := 4;
              Console.WriteLine(Mode(Value)); Console.WriteLine(Mode(in Value));
              Console.WriteLine(Many(1)); Console.WriteLine(Many());
              const Item: Box := new Box(5);
              const Read: function(int): int := Item.Read;
              Console.WriteLine(Read(2));
            end;
            end program;
            """, "int", "double", "generic", "generic", "empty", "int", "value", "in", "fixed", "params", "7");
}
