using Fpas.Compiler.Analysis;
using Fpas.Compiler.Formatting;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;
using Fpas.Compiler.Tests.Support;
using Fpas.Tests.Support;
using System.Text;

namespace Fpas.Compiler.Tests.Syntax;

internal static class BoundaryAssertions
{
    internal static string Render(string atoms, string gap, int? onlyBoundary = null)
    {
        var pieces = atoms.Split('|');
        var text = new StringBuilder(pieces[0]);
        for (var index = 1; index < pieces.Length; index++)
        {
            var separator = onlyBoundary is null || onlyBoundary == index ? gap : " ";
            if (separator.Length == 0 && Word(pieces[index - 1][^1]) && Word(pieces[index][0])) separator = " ";
            // A division followed by an inline comment must not become '///'.
            if (pieces[index - 1] == "/" && separator.StartsWith("//", StringComparison.Ordinal)) separator = " " + separator;
            text.Append(separator).Append(pieces[index]);
        }
        return text.ToString();
    }

    private static bool Word(char value) => char.IsAsciiLetterOrDigit(value) || value == '_';

    internal static string[] Tokens(string source)
    {
        var text = new SourceText("boundary.fpas", source);
        return new Lexer(text).ReadTokens().Where(token => token.Kind != TokenKind.End)
            .Select(token => token.Kind + ":" + source.Substring(token.Position, token.Length)).ToArray();
    }

    internal static void Parses(string source, string context)
    {
        try
        {
            var text = new SourceText("boundary.fpas", source);
            new Parser(text, new Lexer(text).ReadTokens()).ParseCompilationUnit();
            text.Diagnostics.ThrowIfErrors();
        }
        catch (CompilationFailure failure) { Assert.Fail(context + "\n" + string.Join('\n', failure.Diagnostics)); }
    }

    internal static string Formats(string source, IReadOnlyList<string> expected, string context)
    {
        var formatted = FpasFormatter.Format(new("boundary.fpas", source));
        Assert.True(formatted.Success, context + "\n" + string.Join('\n', formatted.Diagnostics));
        Assert.Equal(expected, Tokens(formatted.Text!));
        var repeated = FpasFormatter.Format(new("boundary.fpas", formatted.Text!));
        Assert.True(repeated.Success, context + "\n" + string.Join('\n', repeated.Diagnostics));
        Assert.Equal(formatted.Text, repeated.Text);
        return formatted.Text!;
    }

    internal static void CompilesAndAnalyzes(string source, string context)
    {
        var target = source.StartsWith("unit", StringComparison.Ordinal) ? CompilationTarget.Library : CompilationTarget.Exe;
        var compiled = TestCompilation.Compile(source, target);
        Assert.True(compiled.Success, context + "\n" + string.Join('\n', compiled.Diagnostics));
        var analyzed = FpasCompiler.Analyze(new([new("boundary.fpas", source)], "Boundary",
            [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location], target));
        Assert.True(analyzed.Success, context + "\n" + string.Join('\n', analyzed.Diagnostics));
    }

    internal static void Rejects(string source, string context)
    {
        var formatted = FpasFormatter.Format(new("boundary.fpas", source));
        Assert.False(formatted.Success, context);
        Assert.Null(formatted.Text);
        Assert.NotEmpty(formatted.Diagnostics);
        var compiled = TestCompilation.Compile(source);
        Assert.False(compiled.Success, context);
        Assert.Null(compiled.Program);
        Assert.NotEmpty(compiled.Diagnostics);
    }
}
