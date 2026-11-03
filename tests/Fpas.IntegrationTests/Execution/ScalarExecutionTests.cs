using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class ScalarExecutionTests
{
    [Fact]
    public Task Aliases_and_real_enums_preserve_underlying_types_constants_and_bitwise_values() => ProgramExecution.Output("""
        program Types;
        uses System.Console as Console;
        type Octet = byte;
        type Counter = long;
        type Direction = enum : Octet North; East = 4; West = East + 1; Alias = East; Forward = Last; Last = 8; end enum;
        type Big = enum : ulong Max = 18446744073709551615UL; end enum;
        type Directions = Direction[];
        begin
          const Count: Counter := 42L; Console.WriteLine(Count);
          const Values: Directions := [Direction.North, Direction.West];
          Console.WriteLine(Values[1].ToString()); Console.WriteLine(cast<byte>(Direction.West));
          Console.WriteLine(Direction.East = Direction.Alias); Console.WriteLine(Direction.Forward = Direction.Last);
          Console.WriteLine(cast<byte>(Direction.East or Direction.West)); Console.WriteLine(cast<byte>(not Direction.North));
          Console.WriteLine(cast<ulong>(Big.Max));
          const Unknown: Direction := cast<Direction>(3); Console.WriteLine(cast<byte>(Unknown));
          const Maybe: Direction? := null; Console.WriteLine((Maybe ?? Direction.East) = Direction.East);
        end;
        end program;
        """, "42", "West", "5", "True", "True", "5", "255", "18446744073709551615", "3", "True");

    [Fact]
    public Task Distinct_values_keep_nominal_identity_equality_nullable_and_generic_behavior() => ProgramExecution.Output("""
        program Domains;
        uses System.Console as Console, System.Collections.Generic as C;
        type UserId = distinct long;
        type PlayerName = distinct string;
        type Money = distinct decimal;
        type Flag = distinct bool;
        function Echo<T>(Value: T): T; begin return Value; end function;
        function Ordered(Left: PlayerName; Right: PlayerName): bool;
        requires Left < Right;
        begin return true; end function;
        begin
          const Id: UserId := new UserId(42L); Console.WriteLine(cast<long>(Echo(Id)));
          Console.WriteLine(Id = new UserId(42L)); Console.WriteLine(Id < new UserId(43L));
          const Items: C.HashSet<UserId> := new C.HashSet<UserId>(); discard Items.Add(Id); discard Items.Add(new UserId(42L)); Console.WriteLine(Items.Count);
          const Name: PlayerName := new PlayerName('Ada'); Console.WriteLine(Name.Value); Console.WriteLine(Name = cast<PlayerName>('Ada'));
          const Missing: PlayerName? := null; Console.WriteLine(Missing = null);
          Console.WriteLine(new PlayerName('Ada') < new PlayerName('Zed'));
          const Amount: Money := cast<Money>(1.25M); Console.WriteLine(cast<decimal>(Amount));
          const Default: UserId := new UserId(); Console.WriteLine(Default.Value);
          const Optional: Flag? := cast<Flag?>(true); Console.WriteLine((Optional ?? new Flag(false)).Value);
          const Present: bool? := true; const Lifted: Flag? := cast<Flag?>(Present);
          Console.WriteLine(cast<bool?>(Lifted) ?? false);
          const Absent: bool? := null; Console.WriteLine(cast<Flag?>(Absent) = null);
          Console.WriteLine(Ordered(new PlayerName('Ada'), new PlayerName('Zed')));
        end;
        end program;
        """, "42", "True", "True", "1", "Ada", "True", "True", "True", "1.25", "0", "True", "True", "True", "True");

    [Fact]
    public Task Subrange_defaults_signed_extremes_unsigned_widths_and_arithmetic_remain_valid() => ProgramExecution.Output("""
        program Ranges;
        uses System.Console as Console;
        type Port = ushort range 1..65535;
        type Negative = long range -9223372036854775808L..-1L;
        type Full = long range -9223372036854775808L..9223372036854775807L;
        type High = ulong range 18446744073709551610UL..18446744073709551615UL;
        function Create<T>(): T; where T: struct; begin return new T(); end function;
        begin
          const Ports: Port[] := new Port[2]; Console.WriteLine(Ports[0].Value); Console.WriteLine(Create<Port>().Value);
          const Value: Port := new Port(80); const Number: int := Value; Console.WriteLine(Number); Console.WriteLine(Value + 1);
          Console.WriteLine(Port.LowerBound); Console.WriteLine(Port.UpperBound); Console.WriteLine(Value = new Port(80));
          Console.WriteLine(new Negative().Value); Console.WriteLine(new Negative(-1L).Value);
          Console.WriteLine(new Full(9223372036854775807L).Value);
          const Unsigned: High := new High(18446744073709551615UL); Console.WriteLine(Unsigned.Value); Console.WriteLine(new High().Value);
          const Maybe: Port? := new Port(81); const Lifted: int? := Maybe; Console.WriteLine(Lifted ?? 0);
        end;
        end program;
        """, "1", "1", "80", "81", "1", "65535", "True", "-9223372036854775808", "-1", "9223372036854775807", "18446744073709551615", "18446744073709551610", "81");

    [Fact]
    public Task Wide_numeric_enum_and_exhaustive_subrange_patterns_use_value_comparison() => ProgramExecution.Output("""
        program Matching;
        uses System.Console as Console;
        type High = ulong range 18446744073709551613UL..18446744073709551615UL;
        type Choice = enum : ulong Max = 18446744073709551615UL; end enum;
        function Describe(Value: High): string;
        begin return case Value of
          when 18446744073709551613UL..18446744073709551614UL: 'lower';
          when 18446744073709551615UL: 'upper';
        end case; end function;
        begin
          Console.WriteLine(Describe(new High())); Console.WriteLine(Describe(new High(18446744073709551615UL)));
          const Wide: ulong := 18446744073709551615UL;
          Console.WriteLine(case Wide of when 9223372036854775808UL..18446744073709551615UL: 'unsigned'; else 'small'; end case);
          const Small: byte := 255; Console.WriteLine(case Small of when 255: 'byte'; else 'other'; end case);
          const Price: decimal := 1.25M; Console.WriteLine(case Price of when 1.25M: 'decimal'; else 'other'; end case);
          const Kind: Choice := Choice.Max; Console.WriteLine(case Kind of when Choice.Max: 'enum'; else 'unknown'; end case);
          const Missing: High? := null; Console.WriteLine(case Missing of when null: 'null'; when 18446744073709551613UL..18446744073709551615UL: 'value'; end case);
        end;
        end program;
        """, "lower", "upper", "unsigned", "byte", "decimal", "enum", "null");

    [Theory]
    [InlineData("new Percent(Value)")]
    [InlineData("cast<Percent>(Value)")]
    [InlineData("unchecked(cast<Percent>(Value))")]
    public async Task Runtime_subrange_validation_cannot_be_disabled(string expression)
    {
        var result = await ProgramExecution.Run("program Ranges; type Percent = int range 1..100; begin const Value: int := 101; discard " + expression + "; end; end program;");
        Assert.NotEqual(0, result.ExitCode); Assert.Contains("ArgumentOutOfRangeException", result.Error);
    }
}
