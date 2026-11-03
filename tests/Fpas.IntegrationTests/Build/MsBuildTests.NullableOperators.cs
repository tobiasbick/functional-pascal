using System.Text;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Lifted_operators_match_CLR_results_exceptions_and_evaluation_order(string configuration)
    {
        using var project = sdk.CreateProject();
        (string Name, string Fpas, string CSharp, bool Comparison)[] binary =
        [
            ("Add", "+", "+", false), ("Subtract", "-", "-", false), ("Multiply", "*", "*", false),
            ("Divide", "div", "/", false), ("Remainder", "mod", "%", false), ("And", "and", "&", false),
            ("Or", "or", "|", false), ("Xor", "xor", "^", false), ("ShiftLeft", "shl", "<<", false), ("ShiftRight", "shr", ">>", false),
            ("Equal", "=", "==", true), ("Unequal", "<>", "!=", true), ("Less", "<", "<", true),
            ("Greater", ">", ">", true), ("LessEqual", "<=", "<=", true), ("GreaterEqual", ">=", ">=", true)
        ];
        var source = new StringBuilder("""
            unit Demo.Bridge;
            uses System as Sys, System.Numerics as N, System.Threading.Tasks as Tasks;
            public type Flags = enum : byte None = 0; First = 1; Second = 2; end enum;
            public type Meters = distinct int;
            public type Percent = int range 0..100;
            var Trace: int := 0;
            function Tap(Digit: int; Value: int?): int?;
            begin Trace := Trace * 10 + Digit; return Value; end function;
            public function Evaluate(): int;
            begin
              Trace := 0;
              const Sum: int? := Tap(1, null) + Tap(2, 3);
              if Sum <> null then raise new Sys.InvalidOperationException('Missing result lost'); end if;
              const Skipped: int? := cast<int?>(null) div 0;
              if Skipped <> null then raise new Sys.InvalidOperationException('Division should be absent'); end if;
              try
                discard Tap(3, null) + (1 div 0);
                raise new Sys.InvalidOperationException('Right operand was skipped');
              catch Error: Sys.DivideByZeroException do
              end try;
              return Trace;
            end function;
            public async function AwaitPair(First: Tasks.Task<int?>; Second: Tasks.Task<int?>): Tasks.Task<int?>;
            begin
              return (await First) + (await Second);
            end function;
            public function GenericAdd<T>(A: T?; B: T?): T?;
            where T: struct, N.INumber<T>;
            begin return A + B; end function;
            public function GenericLess<T>(A: T?; B: T?): bool;
            where T: struct, N.INumber<T>;
            begin return A < B; end function;
            public function GenericShift<T>(A: T?; B: int?): T?;
            where T: struct, N.IBinaryInteger<T>;
            begin return A shl B; end function;
            public function UIntConstant(A: uint?): uint?; begin return A + (1 + 2); end function;
            public function UIntVariable(A: uint?; B: int): long?; begin return A + B; end function;
            public function Mixed(A: int?; B: long): long?; begin return A + B; end function;
            public function Small(A: short?; B: byte?): int?; begin return A + B; end function;
            public function Tiny(A: sbyte?; B: ushort?): int?; begin return A + B; end function;
            public function Character(A: char?; B: int?): int?; begin return A + B; end function;
            public function Native(A: nint?; B: int?): nint?; begin return A + B; end function;
            public function NativeUnsigned(A: nuint?): nuint?; begin return A + 1; end function;
            public function NegativeUInt(A: uint?): long?; begin return -A; end function;
            public function Unsigned(A: ulong?; B: ulong?): ulong?; begin return A + B; end function;
            public function Single(A: float?; B: int?): float?; begin return A + B; end function;
            public function Money(A: decimal?; B: decimal?): decimal?; begin return A / B; end function;
            public function FloatEqual(A: double?; B: double?): bool; begin return A = B; end function;
            public function FloatLessEqual(A: double?; B: double?): bool; begin return A <= B; end function;
            public function DateEqual(A: Sys.DateTime?; B: Sys.DateTime?): bool; begin return A = B; end function;
            public function DateLess(A: Sys.DateTime?; B: Sys.DateTime?): bool; begin return A < B; end function;
            public function BoolEqual(A: bool?; B: bool?): bool; begin return A = B; end function;
            public function Negate(A: int?): int?; begin return -A; end function;
            public function Positive(A: short?): int?; begin return +A; end function;
            public function Complement(A: int?): int?; begin return not A; end function;
            public function EnumAnd(A: Flags?; B: Flags?): Flags?; begin return A and B; end function;
            public function EnumNot(A: Flags?): Flags?; begin return not A; end function;
            public function DomainEqual(A: Meters?; B: Meters?): bool; begin return A = B; end function;
            public function RangeAdd(A: Percent?; B: int?): int?; begin return A + B; end function;
            public function Wrapping(A: int?; B: int?): int?; begin return unchecked(A + B); end function;
            public function NullLeft(A: int?): int?; begin return null + A; end function;
            public function NullRight(A: int?): int?; begin return A + null; end function;
            public function NullShift(A: long?): long?; begin return A shl null; end function;
            public function SourceCalls(): bool;
            begin return GenericAdd<int>(1, 2) = 3 and GenericAdd<int>(null, 2) = null and
              not GenericLess<int>(null, 1) and GenericShift<int>(3, 1) = 6; end function;
            public function Contract(A: int?; B: int?): int?;
            requires A + B = 3;
            begin return A + B; end function;
            """);
        foreach (var item in binary)
            source.AppendLine($"public function {item.Name}(A: int?; B: int?): {(item.Comparison ? "bool" : "int?")}; begin return A {item.Fpas} B; end function;");
        source.AppendLine("end unit;");
        File.WriteAllText(project.File("Game/Bridge.fpas"), source.ToString());
        var checks = string.Join('\n', binary.Select(item => $"Check(() => Bridge.{item.Name}(a, b), () => checked(a {item.CSharp} b));"));
        File.WriteAllText(project.File("Host/Program.cs"), $$"""
            using Demo;
            int?[] values = [null, int.MinValue, -2, 0, 2, 33, int.MaxValue];
            foreach (var a in values)
            {
                Check(() => Bridge.Negate(a), () => checked(-a));
                Check(() => Bridge.Complement(a), () => ~a);
                foreach (var b in values)
                {
                    {{checks}}
                    Check(() => Bridge.Wrapping(a, b), () => unchecked(a + b));
                    Check(() => Bridge.GenericAdd(a, b), () => checked(a + b));
                    Check(() => Bridge.GenericLess(a, b), () => a < b);
                    Check(() => Bridge.GenericShift(a, b), () => a << b);
                }
            }
            foreach (uint? a in new uint?[] { null, 1, uint.MaxValue })
            {
                Check(() => Bridge.UIntConstant(a), () => checked(a + (1 + 2)));
                Check(() => Bridge.UIntVariable(a, -1), () => checked(a + -1));
            }
            Check(() => Bridge.Mixed(2, long.MaxValue), () => checked((int?)2 + long.MaxValue));
            Check(() => Bridge.Mixed(null, 3), () => (int?)null + 3L);
            Check(() => Bridge.Small(2, 3), () => (short?)2 + (byte?)3);
            Check(() => Bridge.Small(null, 3), () => (short?)null + (byte?)3);
            Check(() => Bridge.Tiny(-1, ushort.MaxValue), () => (sbyte?)-1 + (ushort?)ushort.MaxValue);
            Check(() => Bridge.Character('A', 1), () => (char?)'A' + 1);
            Check(() => Bridge.Positive(2), () => +(short?)2);
            Check(() => Bridge.Native(nint.MaxValue, 1), () => checked((nint?)nint.MaxValue + 1));
            Check(() => Bridge.Native(null, 1), () => (nint?)null + 1);
            Check(() => Bridge.NativeUnsigned(nuint.MaxValue), () => checked((nuint?)nuint.MaxValue + 1));
            Check(() => Bridge.NativeUnsigned(null), () => (nuint?)null + 1);
            Check(() => Bridge.NegativeUInt(uint.MaxValue), () => -(uint?)uint.MaxValue);
            Check(() => Bridge.Unsigned(ulong.MaxValue, 1), () => checked((ulong?)ulong.MaxValue + 1));
            Check(() => Bridge.Single(1.5f, 2), () => (float?)1.5f + 2);
            foreach (decimal? a in new decimal?[] { null, 0, 1, decimal.MaxValue })
                foreach (decimal? b in new decimal?[] { null, 0, 3, 0.1m })
                    Check(() => Bridge.Money(a, b), () => a / b);
            foreach (double? a in new double?[] { null, double.NaN, double.PositiveInfinity, -0.0, 1 })
                foreach (double? b in new double?[] { null, double.NaN, double.NegativeInfinity, 0.0, 2 })
                {
                    Check(() => Bridge.FloatEqual(a, b), () => a == b);
                    Check(() => Bridge.FloatLessEqual(a, b), () => a <= b);
                }
            foreach (DateTime? a in new DateTime?[] { null, DateTime.MinValue, DateTime.MaxValue })
                foreach (DateTime? b in new DateTime?[] { null, DateTime.MinValue, DateTime.MaxValue })
                {
                    Check(() => Bridge.DateEqual(a, b), () => a == b);
                    Check(() => Bridge.DateLess(a, b), () => a < b);
                }
            foreach (bool? a in new bool?[] { null, false, true })
                foreach (bool? b in new bool?[] { null, false, true }) Check(() => Bridge.BoolEqual(a, b), () => a == b);
            Check(() => Bridge.NullLeft(1), () => (int?)null);
            Check(() => Bridge.NullRight(1), () => (int?)null);
            Check(() => Bridge.NullShift(1), () => (long?)null);
            Check(() => Bridge.EnumAnd(Bridge.Flags.First, Bridge.Flags.Second), () => (Bridge.Flags?)Bridge.Flags.None);
            Check(() => Bridge.EnumAnd(null, Bridge.Flags.First), () => (Bridge.Flags?)null);
            Check(() => Bridge.EnumNot(Bridge.Flags.First), () => (Bridge.Flags?)unchecked((Bridge.Flags)(byte)~(byte)Bridge.Flags.First));
            Check(() => Bridge.EnumNot(null), () => (Bridge.Flags?)null);
            Check(() => Bridge.DomainEqual(new Bridge.Meters(2), new Bridge.Meters(2)), () => true);
            Check(() => Bridge.DomainEqual(null, null), () => true);
            Check(() => Bridge.DomainEqual(null, new Bridge.Meters(2)), () => false);
            Check(() => Bridge.RangeAdd(new Bridge.Percent(50), 100), () => (int?)150);
            Check(() => Bridge.RangeAdd(null, 100), () => (int?)null);
            Check(() => Bridge.Contract(1, 2), () => (int?)3);
            Check(() => Bridge.Contract(null, 2), () => throw new InvalidOperationException());
            if (!Bridge.SourceCalls() || Bridge.Evaluate() != 123) throw new Exception("Source calls or operand order changed.");
            foreach (int? first in new int?[] { null, 2 })
            {
                var second = new TaskCompletionSource<int?>();
                var result = Bridge.AwaitPair(Task.FromResult(first), second.Task);
                if (result.IsCompleted) throw new Exception("Right operand did not suspend.");
                second.SetResult(3);
                if (await result != first + 3) throw new Exception("Left operand was lost across suspension.");
            }
            var failedSecond = new TaskCompletionSource<int?>();
            var failedResult = Bridge.AwaitPair(Task.FromResult((int?)null), failedSecond.Task);
            if (failedResult.IsCompleted) throw new Exception("Missing left operand skipped the right await.");
            failedSecond.SetException(new DivideByZeroException());
            try { await failedResult; throw new Exception("Right operand fault was suppressed."); }
            catch (DivideByZeroException) { }
            Console.WriteLine("Nullable operators checked");
            static void Check<T>(Func<T> actual, Func<T> expected)
            {
                var a = Observe(actual); var e = Observe(expected);
                if (a.Error != e.Error || a.Error is null && !EqualityComparer<T>.Default.Equals(a.Value, e.Value))
                    throw new Exception($"Nullable result differs: actual {a}, expected {e}");
            }
            static (T? Value, Type? Error) Observe<T>(Func<T> operation)
            {
                try { return (operation(), null); }
                catch (Exception error) { return (default, error.GetType()); }
            }
            """);
        await BuildAndRunContractProject(project, configuration, "Nullable operators checked");
    }
}
