using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

internal static class UsageTypeArguments
{
    internal static TypeSymbol Resolve(string spelling, ReferenceCatalog references, CancellationToken token)
    {
        var source = new SourceText("type argument", spelling, token);
        var tokens = new Lexer(source).ReadTokens();
        var type = new Parser(source, tokens).ParseStandaloneType();
        // A standalone type has no surrounding uses clause. Resolve roots under their own names.
        var imports = tokens.Where(t => t.Kind == TokenKind.Identifier).DistinctBy(t => t.Text)
            .Select(root => new ImportSyntax(new([root]), root)).ToArray();
        var unitName = new NameSyntax([new Token(TokenKind.Identifier, "ApiUsage", 0, 0)]);
        var unit = new CompilationUnitSyntax(unitName, IsProgram: false, imports, [], [], [], []);
        return new SourceCatalog().ResolveType(type, new SourceUnit(source, unit), references);
    }
}
