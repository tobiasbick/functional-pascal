using System.Text.Json;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    private const string GenericMathReference = """
        using System.Numerics;
        namespace Shared;
        public readonly record struct Number(int Value) : IAdditionOperators<Number, Number, Number>, IUnaryNegationOperators<Number, Number>
        {
            public static Number operator +(Number left, Number right) => new(unchecked(left.Value + right.Value));
            public static Number operator checked +(Number left, Number right) => new(checked(left.Value + right.Value));
            public static Number operator -(Number value) => new(unchecked(-value.Value));
            public static Number operator checked -(Number value) => new(checked(-value.Value));
        }
        public sealed record ReferenceNumber(int Value) : IAdditionOperators<ReferenceNumber, ReferenceNumber, ReferenceNumber>
        {
            public static ReferenceNumber operator +(ReferenceNumber left, ReferenceNumber right) => new(left.Value + right.Value);
        }
        public readonly record struct Offset(long Value) : IAdditionOperators<Offset, int, long>
        {
            public static long operator +(Offset left, int right) => left.Value + right;
        }
        public interface IReverse<T> where T : IReverse<T>
        {
            static abstract long operator +(int left, T right);
        }
        public readonly record struct Reverse(long Value) : IReverse<Reverse>
        {
            public static long operator +(int left, Reverse right) => left + right.Value;
        }
        public readonly record struct Probe(int Value) : IAdditionOperators<Probe, Probe, int>
        {
            public static string Log = "";
            public static Probe Left() { Log += "L"; return new(20); }
            public static Probe Right() { Log += "R"; return new(22); }
            public static int operator +(Probe left, Probe right) { Log += "+"; return left.Value + right.Value; }
        }
        public interface IDefault<T> where T : IDefault<T>
        {
            static virtual int Value => 17;
            static virtual T Echo(T value) => value;
        }
        public interface IOverride<T> : IDefault<T> where T : IOverride<T>
        {
            static int IDefault<T>.Value => 23;
        }
        public sealed class Default : IOverride<Default> { }
        public interface ICommon<T> where T : ICommon<T> { static abstract int operator +(T left, int right); }
        public interface ILeft<T> : ICommon<T> where T : ILeft<T> { }
        public interface IRight<T> : ICommon<T> where T : IRight<T> { }
        public interface IDiamond<T> : ILeft<T>, IRight<T> where T : IDiamond<T> { }
        public readonly struct Diamond : IDiamond<Diamond> { public static int operator +(Diamond left, int right) => right; }
        public interface IFirst<T> where T : IFirst<T> { static abstract int operator +(T left, int right); }
        public interface ISecond<T> where T : ISecond<T> { static abstract int operator +(T left, int right); }
        public interface IFirstProperty { static abstract int Value { get; } }
        public interface ISecondProperty { static abstract int Value { get; } }
        public static class Oracle
        {
            public static T Calculate<T>(T left, T right) where T : INumber<T> => (left * right + left - right) / T.CreateChecked(2);
        }
        """;

    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Constrained_static_interface_operators_and_members_execute_across_CLR_boundaries(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Math.cs"), GenericMathReference);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Math;
            uses System.Numerics as N, System.Threading.Tasks as Tasks, Shared as S;
            public function Add<T>(Left: T; Right: T): T;
            where T: N.IAdditionOperators<T, T, T>;
            begin return Left + Right; end function;
            public function Wrap<T>(Left: T; Right: T): T;
            where T: N.IAdditionOperators<T, T, T>;
            begin return unchecked(Left + Right); end function;
            public function Negate<T>(Value: T): T;
            where T: N.IUnaryNegationOperators<T, T>;
            begin return -Value; end function;
            public function WrapNegate<T>(Value: T): T;
            where T: N.IUnaryNegationOperators<T, T>;
            begin return unchecked(-Value); end function;
            public function Positive<T>(Value: T): T;
            where T: N.IUnaryPlusOperators<T, T>;
            begin return +Value; end function;
            public function Calculate<T>(Left: T; Right: T): T;
            where T: N.INumber<T>;
            begin return (Left * Right + Left - Right) / T.CreateChecked(2); end function;
            public function Bits<T>(Left: T; Right: T; Count: int): T;
            where T: N.IBinaryInteger<T>;
            begin return ((not Left and Right) or ((Left xor Right) shl Count)) shr 1; end function;
            public function Divide<T>(Left: T; Right: T): T;
            where T: N.IDivisionOperators<T, T, T>;
            begin return Left div Right; end function;
            public function Remainder<T>(Left: T; Right: T): T;
            where T: N.IModulusOperators<T, T, T>;
            begin return Left mod Right; end function;
            public function Compare<T>(Left: T; Right: T): bool;
            where T: N.IComparisonOperators<T, T, bool>;
            begin return Left < Right and Right > Left and Left <= Right and Right >= Left and Left <> Right and Left = Left; end function;
            public function Zero<T>(): T;
            where T: N.INumber<T>;
            begin return T.Zero; end function;
            public function Convert<T>(Value: int): T;
            where T: N.INumberBase<T>;
            begin return T.CreateChecked(Value); end function;
            public function Converter<T>(): function(int): T;
            where T: N.INumberBase<T>;
            begin return T.CreateChecked<int>; end function;
            public function Capture<T>(Start: T): function(T): T;
            where T: N.INumber<T>;
            begin return function(Value: T): T => Start + Value; end function;
            public type Accumulator<T> = where T: N.INumber<T>;
            record
              Value: T;
              function Add(Other: T): T; begin return self.Value + Other; end function;
            end record;
            public type DefaultRecord = record implements S.IDefault<DefaultRecord> end record;
            public function DefaultValue<T>(): int;
            where T: S.IDefault<T>;
            begin return T.Value; end function;
            public function DefaultEcho<T>(Value: T): T;
            where T: S.IDefault<T>;
            begin return T.Echo(Value); end function;
            public function Mixed<TLeft, TRight, TResult>(Left: TLeft; Right: TRight): TResult;
            where TLeft: N.IAdditionOperators<TLeft, TRight, TResult>;
            begin return Left + Right; end function;
            public function Reverse<T>(Left: int; Right: T): long;
            where T: S.IReverse<T>;
            begin return Left + Right; end function;
            public function Diamond<T>(Value: T): int;
            where T: S.IDiamond<T>;
            begin return Value + 42; end function;
            public function Order(): int;
            begin return Mixed<S.Probe, S.Probe, int>(S.Probe.Left(), S.Probe.Right()); end function;
            public async function Later<T>(First: T; Pending: Tasks.Task<T>): Tasks.Task<T>;
            where T: N.INumber<T>;
            begin return First + await Pending; end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Consumer"));
        File.WriteAllText(project.File("Consumer/Consumer.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Consumer/Consumer.fpas"), """
            unit Demo.Consumer;
            uses Demo.Math as M, System.Numerics as N;
            public function Forward<T>(Left: T; Right: T): T;
            where T: N.INumber<T>;
            begin return M.Add(Left, Right); end function;
            end unit;
            """);
        var host = XDocument.Load(project.File("Host/Host.csproj"));
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj"))));
        host.Save(project.File("Host/Host.csproj"));
        File.WriteAllText(project.File("Host/Program.cs"), """
            using M = Demo.Math;
            using Shared;
            using System.Numerics;
            if (M.Add(20, 22) != 42 || M.Add(1.25m, 2.5m) != 3.75m || M.Add(new BigInteger(40), new BigInteger(2)) != 42) throw new Exception("Generic addition");
            if (M.Calculate(9, 3) != Oracle.Calculate(9, 3) || M.Calculate(9.5, 3.0) != Oracle.Calculate(9.5, 3.0) || M.Calculate(9.5m, 3m) != Oracle.Calculate(9.5m, 3m)) throw new Exception("Inherited numeric contracts");
            if (M.Add(new ReferenceNumber(40), new ReferenceNumber(2)).Value != 42 || M.Mixed<Offset, int, long>(new(40), 2) != 42 || M.Reverse(2, new Reverse(40)) != 42) throw new Exception("Operand and result identities");
            if (M.Bits(12, 7, 1) != ((~12 & 7 | ((12 ^ 7) << 1)) >> 1) || M.Divide(9, 2) != 4 || M.Remainder(9, 2) != 1 || !M.Compare(3, 7) || M.Compare(7, 3) || M.Compare(double.NaN, 1.0)) throw new Exception("Bitwise, division and comparisons");
            if (M.Positive(7) != 7 || M.Negate(7) != -7 || M.Zero<int>() != 0 || M.Zero<decimal>() != 0m || M.Convert<BigInteger>(42) != 42 || M.Converter<decimal>()(42) != 42m) throw new Exception("Static properties, generic calls and method groups");
            if (M.Capture(20)(22) != 42 || new M.Accumulator<int>(20).Add(22) != 42 || Demo.Consumer.Forward(20, 22) != 42) throw new Exception("Generic environments and exported constraints");
            if (M.Diamond(new Diamond()) != 42 || M.DefaultValue<Default>() != 23 || M.DefaultValue<M.DefaultRecord>() != 17) throw new Exception("Diamond or most-specific default dispatch");
            var reference = new Default();
            if (!ReferenceEquals(reference, M.DefaultEcho(reference))) throw new Exception("Static virtual default method");
            Probe.Log = "";
            if (M.Order() != 42 || Probe.Log != "LR+") throw new Exception("Operands and operator execute once in order");
            try { M.Add(int.MaxValue, 1); throw new Exception("Missing checked Int32 addition"); } catch (OverflowException) { }
            try { M.Add(new Number(int.MaxValue), new Number(1)); throw new Exception("Missing checked custom addition"); } catch (OverflowException) { }
            try { M.Negate(new Number(int.MinValue)); throw new Exception("Missing checked custom negation"); } catch (OverflowException) { }
            if (M.Wrap(int.MaxValue, 1) != int.MinValue || M.Wrap(new Number(int.MaxValue), new Number(1)).Value != int.MinValue || M.WrapNegate(new Number(int.MinValue)).Value != int.MinValue) throw new Exception("Unchecked operators");
            try { M.Add(decimal.MaxValue, 1m); throw new Exception("Missing decimal overflow"); } catch (OverflowException) { }
            var pending = new TaskCompletionSource<int>(TaskCreationOptions.RunContinuationsAsynchronously);
            var task = M.Later(20, pending.Task);
            if (task.IsCompleted) throw new Exception("No real async suspension");
            pending.SetResult(22);
            if (await task != 42) throw new Exception("Static dispatch after await");
            Console.WriteLine("Generic math verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "--configuration", configuration, "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, $"Host/bin/{configuration}/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Generic math verified" + Environment.NewLine, run.Output);
    }

    [Fact]
    public async Task Ambiguous_or_unconstrained_static_interface_access_fails_before_emission()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Math.cs"), GenericMathReference);
        var build = await sdk.Run(project, "build", "Shared/Shared.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        (string Source, string Code)[] cases =
        [
            ("function F<T>(X: T): int; where T: S.IFirst<T>, S.ISecond<T>; begin return X + 1; end function;", "FP3003"),
            ("function F<T>(): int; where T: S.IFirstProperty, S.ISecondProperty; begin return T.Value; end function;", "FP3003"),
            ("function F<T>(X: T): T; where T: N.IAdditionOperators<T, T, T>; begin return X * X; end function;", "FP3002"),
            ("function F(): int; begin return S.IFirstProperty.Value; end function;", "FP3005"),
            ("function F(): int; begin return S.IDefault<S.Default>.Echo(new S.Default()).GetHashCode(); end function;", "FP3005"),
            ("function F<T>(X: T; Y: T): T; where T: N.IAdditionOperators<T, T, T>; begin return X + Y; end function; function Bad(): string; begin return F<string>('a', 'b'); end function;", "FP3018"),
            ("type Bad = record implements S.IFirstProperty end record;", "FP3029")
        ];
        foreach (var (declaration, code) in cases)
        {
            var source = "program Invalid; uses Shared as S, System.Numerics as N; " + declaration + " begin end; end program;";
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Invalid.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"], source);
            Assert.NotEqual(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            var diagnostic = Assert.Single(document.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.True(code == diagnostic.GetProperty("code").GetString(), declaration + ": " + diagnostic.GetProperty("message").GetString());
            Assert.False(File.Exists(project.File("Invalid.dll")));
        }
    }
}
