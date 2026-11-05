namespace Fpas.Compiler.Tests.Formatting;

public sealed class SourceSectionTests
{
    [Theory]
    [InlineData("\n")]
    [InlineData("\r\n")]
    [InlineData("\r")]
    public void Authored_binding_sections_survive_without_turning_comments_into_sections(string newline)
    {
        const string input = """
            program Sections;
            const Minimum: int := 1;
            const Maximum: int := 2;


            const Limit: int := Minimum + Maximum;
            begin
            const First: int := 1;
            const Second: int := 2;


            const Sum: int := First + Second;

            // Track the computed result.
            var Seen: int := Sum;
            Seen := Seen + 1; // Update the same result.
            // Keep these related bindings together.
            // A second comment is still not a blank line.
            const Copy: int := Seen;
            // This comment deliberately stays detached.

            const Detached: int := Copy;

            discard Detached + Limit;
            end;
            end program;
            """;
        const string expected = """
            program Sections;

            const Minimum: int := 1;
            const Maximum: int := 2;

            const Limit: int := Minimum + Maximum;

            begin
              const First: int := 1;
              const Second: int := 2;

              const Sum: int := First + Second;

              // Track the computed result.
              var Seen: int := Sum;
              Seen := Seen + 1; // Update the same result.
              // Keep these related bindings together.
              // A second comment is still not a blank line.
              const Copy: int := Seen;
              // This comment deliberately stays detached.

              const Detached: int := Copy;

              discard Detached + Limit;
            end;
            end program;
            """;
        var source = input.Replace("\r\n", "\n", StringComparison.Ordinal).Replace("\n", newline, StringComparison.Ordinal);
        Assert.Equal(expected.Replace("\r\n", "\n", StringComparison.Ordinal) + "\n", FormatterTests.Format(source));
    }
}
