namespace Fpas.Compiler.Tests.Syntax;

public sealed class TokenBoundaryTests
{
    public static IEnumerable<object[]> Cases() =>
        from program in BoundaryCases.Programs
        from gap in BoundaryCases.Gaps
        select new object[] { program.Name, gap.Name, program.Atoms, gap.Text };

    [Theory]
    [MemberData(nameof(Cases))]
    public void One_boundary_at_a_time_accepts_optional_trivia_and_preserves_tokens(string name, string layout, string atoms, string gap)
    {
        var baseline = BoundaryAssertions.Render(atoms, " ");
        var expected = BoundaryAssertions.Tokens(baseline);
        var boundaryCount = atoms.Count(character => character == '|');
        for (var boundary = 1; boundary <= boundaryCount; boundary++)
        {
            var context = $"{name}, {layout}, boundary {boundary}";
            var source = BoundaryAssertions.Render(atoms, gap, boundary);
            Assert.Equal(expected, BoundaryAssertions.Tokens(source));
            BoundaryAssertions.Parses(source, context);
            BoundaryAssertions.Formats(source, expected, context);
        }
    }

    [Theory]
    [MemberData(nameof(Cases))]
    public void All_boundaries_together_preserve_compilation_analysis_and_formatting(string name, string layout, string atoms, string gap)
    {
        var context = name + ", " + layout;
        var expected = BoundaryAssertions.Tokens(BoundaryAssertions.Render(atoms, " "));
        var source = BoundaryAssertions.Render(atoms, gap);
        Assert.Equal(expected, BoundaryAssertions.Tokens(source));
        BoundaryAssertions.CompilesAndAnalyzes(source, context);
        var formatted = BoundaryAssertions.Formats(source, expected, context);
        BoundaryAssertions.CompilesAndAnalyzes(formatted, context + ", formatted");
    }
}
