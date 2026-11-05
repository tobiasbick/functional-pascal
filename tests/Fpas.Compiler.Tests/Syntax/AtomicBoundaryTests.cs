namespace Fpas.Compiler.Tests.Syntax;

public sealed class AtomicBoundaryTests
{
    public static IEnumerable<object[]> Cases() =>
        from item in BoundaryCases.Atomic
        from gap in BoundaryCases.Gaps.Where(gap => gap.Text.Length != 0)
        select new object[] { item.Name, gap.Name, item.Prefix, item.Atom, item.Suffix, gap.Text };

    [Theory]
    [MemberData(nameof(Cases))]
    public void Operators_literals_and_escapes_cannot_be_split_or_repaired(string name, string layout, string prefix, string atom, string suffix, string gap)
    {
        var context = name + ", " + layout;
        var original = prefix + atom + suffix;
        BoundaryAssertions.CompilesAndAnalyzes(original, context + ", original");
        BoundaryAssertions.Formats(original, BoundaryAssertions.Tokens(original), context + ", original");
        for (var split = 1; split < atom.Length; split++)
            BoundaryAssertions.Rejects(prefix + atom[..split] + gap + atom[split..] + suffix, context + $", split {split}");
    }

    [Theory]
    [InlineData("program Boundary;begin if true then return;end", "if", ";end;end program;")]
    [InlineData("program Boundary;begin end;end", "program", ";")]
    [InlineData("unit Boundary;end", "unit", ";")]
    [InlineData("program", "Boundary", ";begin end;end program;")]
    [InlineData("program Boundary;const", "Value", ":int:=1;begin end;end program;")]
    public void Required_word_separators_are_preserved(string prefix, string next, string suffix)
    {
        foreach (var gap in BoundaryCases.Gaps)
        {
            var source = prefix + gap.Text + next + suffix;
            if (gap.Text.Length == 0) BoundaryAssertions.Rejects(source, prefix + next);
            else BoundaryAssertions.CompilesAndAnalyzes(source, gap.Name);
        }
    }
}
