using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Comparison_overloads_consider_both_owners_preserve_order_and_reject_ambiguity()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Comparisons.cs"), """
            namespace Shared;
            public readonly struct Measure(int value)
            {
                public int Value => value;
                public static bool operator <(Measure left, long right) => left.Value < right;
                public static bool operator >(Measure left, long right) => left.Value > right;
                public static bool operator <(Measure left, double right) => throw new Exception("Wrong overload");
                public static bool operator >(Measure left, double right) => throw new Exception("Wrong overload");
                public static bool operator <(long left, Measure right) => left < right.Value;
                public static bool operator >(long left, Measure right) => left > right.Value;
            }
            public sealed class Left
            {
                public static bool operator <(Left left, Right right) => true;
                public static bool operator >(Left left, Right right) => false;
            }
            public sealed class Right
            {
                public static bool operator <(Left left, Right right) => false;
                public static bool operator >(Left left, Right right) => true;
            }
            public sealed class EqualValue(int value)
            {
                public static int Calls;
                public static bool operator ==(EqualValue? left, EqualValue? right)
                { Calls++; return left?.Value == right?.Value; }
                public static bool operator !=(EqualValue? left, EqualValue? right) => !(left == right);
                public int Value => value;
                public override bool Equals(object? other) => other is EqualValue item && value == item.Value;
                public override int GetHashCode() => value;
            }
            """);
        var result = await RunInterop(project, """
            program Comparisons;
            uses Shared as S, System.Console as Console;
            var Order: int := 0;
            function LeftValue(): S.Measure;
            begin Order := Order * 10 + 1; return new S.Measure(4); end function;
            function RightValue(): int;
            begin Order := Order * 10 + 2; return 5; end function;
            function Read(Value: S.EqualValue?): int;
            begin if Value is S.EqualValue then return Value.Value; else return 0; end if; end function;
            begin
              Console.WriteLine(LeftValue() < RightValue());
              Console.WriteLine(Order);
              Console.WriteLine(3 < new S.Measure(4));
              const First: S.EqualValue := new S.EqualValue(7);
              const Second: S.EqualValue := new S.EqualValue(7);
              Console.WriteLine(First = Second);
              Console.WriteLine(First <> null);
              Console.WriteLine(null = First);
              Console.WriteLine(S.EqualValue.Calls);
              Console.WriteLine(Read(First));
              Console.WriteLine(Read(null));
            end;
            end program;
            """);
        Assert.Equal(0, result.ExitCode);
        Assert.Equal(string.Join(Environment.NewLine, "True", "12", "True", "True", "True", "False", "3", "7", "0", ""), result.Output);
        var ambiguous = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Ambiguous.dll"),
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll")], """
            program Ambiguous;
            uses Shared as S;
            begin discard new S.Left() < new S.Right(); end;
            end program;
            """);
        Assert.NotEqual(0, ambiguous.ExitCode);
        Assert.Contains("FP3003", ambiguous.Error);
        Assert.Contains("Ambiguous", ambiguous.Error);
        Assert.False(File.Exists(project.File("Ambiguous.dll")));
        foreach (var body in new[] { "if Value <> null then discard Value.Value; end if;", "discard Value <> null and Value.Value > 0;" })
        {
            var unsafeGuard = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Unsafe.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll")],
                "program Unsafe; uses Shared as S; procedure Read(Value: S.EqualValue?); begin " + body + " end procedure; begin end; end program;");
            Assert.NotEqual(0, unsafeGuard.ExitCode);
            Assert.Contains("FP3013", unsafeGuard.Error);
        }
    }
}
