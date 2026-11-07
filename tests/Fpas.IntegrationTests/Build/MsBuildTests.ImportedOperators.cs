using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Imported_operators_bind_both_owners_checked_overloads_and_lifted_results()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Operators.cs"), """
            namespace Shared;

            public readonly struct Measure(int value)
            {
                public static int Calls;
                public int Value => value;

                public static Measure operator +(Measure left, long right) => new(left.Value + (int)right);
                public static Measure operator +(Measure left, double right) => throw new Exception("Wrong overload");
                public static Measure operator checked +(Measure left, long right)
                {
                    Calls++;
                    return new(left.Value + (int)right + 100);
                }
                public static Measure operator +(long left, Measure right) => new((int)left + right.Value);
                public static int operator -(Measure left, Measure right) => left.Value - right.Value;
                public static Measure operator -(Measure item) => new(-item.Value);
                public static Measure operator checked -(Measure item) => new(100 - item.Value);
                public static int operator /(Measure left, int right) => left.Value / right;
                public static Measure operator &(Measure left, Measure right) => new(left.Value & right.Value);
                public static Measure operator &(bool left, Measure right) => right;
                public static bool operator !(Measure item) => item.Value == 0;
            }

            public sealed class Left
            {
                public static int operator +(Left left, Right right) => 1;
            }

            public sealed class Right
            {
                public static int operator +(Left left, Right right) => 2;
            }

            public sealed class OrdinaryMethod
            {
                public static int op_Addition(OrdinaryMethod left, OrdinaryMethod right) => 3;
            }

            public readonly struct Box<T>(T value)
            {
                public static T operator +(Box<T> left, Box<T> right) => right.Read();
                public T Read() => value;
            }
            """);
        var result = await RunInterop(project, """
            program ImportedOperators;
            uses Shared as S, System.Console as Console;

            var Order: int := 0;

            function LeftValue(): S.Measure;
            begin Order := Order * 10 + 1; return new S.Measure(4); end function;
            function RightValue(): int;
            begin Order := Order * 10 + 2; return 5; end function;
            function MissingValue(): S.Measure?;
            begin Order := Order * 10 + 3; return null; end function;
            function Maybe(Value: S.Measure): S.Measure?;
            begin return Value; end function;

            begin
              const Checked: S.Measure := LeftValue() + RightValue();
              const Ordinary: S.Measure := unchecked(new S.Measure(4) + 5);
              const FromRight: S.Measure := 3 + new S.Measure(4);
              const Negative: S.Measure := -new S.Measure(4);
              const OrdinaryNegative: S.Measure := unchecked(-new S.Measure(4));
              const Bitwise: S.Measure := new S.Measure(6) and new S.Measure(3);
              Console.WriteLine(Checked.Value);
              Console.WriteLine(Ordinary.Value);
              Console.WriteLine(FromRight.Value);
              Console.WriteLine(Negative.Value);
              Console.WriteLine(OrdinaryNegative.Value);
              Console.WriteLine(Bitwise.Value);
              Console.WriteLine(not new S.Measure(0));
              Console.WriteLine(new S.Measure(9) / 2);
              Console.WriteLine(new S.Measure(9) div 2);
              Console.WriteLine(new S.Box<int>(1) + new S.Box<int>(42));

              const Present: S.Measure? := Maybe(new S.Measure(4)) + 5;
              const Absent: S.Measure? := MissingValue() + RightValue();
              const PresentValue: S.Measure := Present ?? new S.Measure(-1);
              Console.WriteLine(PresentValue.Value);
              Console.WriteLine(Absent = null);
              Console.WriteLine(Order);
              Console.WriteLine(S.Measure.Calls);
            end;
            end program;
            """);
        Assert.Equal(0, result.ExitCode);
        Assert.Equal(string.Join(Environment.NewLine, "109", "9", "7", "96", "-4", "2", "True", "4", "4", "42",
            "109", "True", "1232", "2", ""), result.Output);

        var output = project.File("Invalid.dll");
        foreach (var (source, diagnostic) in new[]
        {
            ("begin discard new S.Left() + new S.Right(); end;", "FP3003"),
            ("begin discard new S.OrdinaryMethod() + new S.OrdinaryMethod(); end;", "FP3002"),
            ("begin discard false and new S.Measure(4); end;", "FP3002"),
            ("function F(A: S.Measure; B: S.Measure): bool; requires A - B > 0; begin return true; end function; begin end;", "FP3027")
        })
        {
            var invalid = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", output,
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll")],
                $"program Invalid; uses Shared as S; {source} end program;");
            Assert.NotEqual(0, invalid.ExitCode);
            Assert.Contains(diagnostic, invalid.Error);
            Assert.False(File.Exists(output));
        }
    }
}
