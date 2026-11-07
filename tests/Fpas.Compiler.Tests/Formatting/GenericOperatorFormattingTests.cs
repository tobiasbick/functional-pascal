using Fpas.Compiler.Formatting;

namespace Fpas.Compiler.Tests.Formatting;

public sealed class GenericOperatorFormattingTests
{
    [Fact]
    public void Array_type_spelling_keeps_rectangular_rank_and_nullable_groups()
    {
        var output = FormatterTests.Format("program Demo;begin const Values:(array [ , ] of string ? ) ? :=null;end;end program;");
        Assert.Contains("const Values: (array[,] of string?)? := null;", output);
    }

    [Fact]
    public void Generic_closers_and_comparison_operators_get_their_contextual_spacing()
    {
        var output = FormatterTests.Format("program Demo;type Box<T>=record public Value:T;end record;begin const X:int:=(1 and 3)or(2 xor 1);if X>=1 then discard X;end if;end;end program;");
        Assert.Contains("type Box<T> = record", output);
        Assert.Contains("Value: T;", output);
        Assert.Contains("const X: int := (1 and 3) or (2 xor 1);", output);
        Assert.Contains("if X >= 1 then", output);
        Assert.DoesNotContain("if X > =", output);
    }

    [Fact]
    public void A_comparison_wraps_after_the_whole_operator()
    {
        var left = new string('A', 65); var right = new string('B', 45);
        var output = FormatterTests.Format($"program Demo;begin if {left}>={right} then return;end if;end;end program;");
        Assert.Contains(left + " >=\n", output);
        Assert.DoesNotContain(">\n", output);
        Assert.All(output.Split('\n'), line => Assert.True(line.Length <= 100, line));
    }

    [Theory]
    [InlineData("X>=\n1")]
    [InlineData("X\n>=1")]
    public void Authored_comparison_breaks_keep_the_operator_intact(string condition)
    {
        var output = FormatterTests.Format($"program Demo;begin if {condition} then return;end if;end;end program;");
        Assert.Contains("X >=\n", output);
        Assert.DoesNotContain("> =", output);
    }

    [Fact]
    public void Split_comparison_operators_produce_diagnostics_instead_of_repaired_source()
    {
        var result = FpasFormatter.Format(new("invalid.fpas", "program Demo;begin if 2 > = 1 then return;end if;end;end program;"));
        Assert.False(result.Success); Assert.Null(result.Text);
        Assert.Contains(result.Diagnostics, diagnostic => diagnostic.Code == "FP2003");
    }
}
