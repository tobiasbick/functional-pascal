using Fpas.Compiler.Analysis;
using Fpas.LanguageServer.Protocol;
using Newtonsoft.Json.Linq;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private static readonly string[] SyntaxMethods = ["textDocument/documentSymbol", "textDocument/foldingRange", "textDocument/selectionRange"];
    private bool hierarchicalSymbols;

    private object SyntaxFeature(string method, JObject parameters, RequestStamp stamp, CancellationToken token)
    {
        var open = OpenDocument(DocumentPath(parameters), stamp);
        var syntax = SyntaxAnalysis.Structure(new(open.Path, open.Text), token);
        return method switch
        {
            "textDocument/documentSymbol" => DocumentSymbols(syntax, hierarchicalSymbols, token),
            "textDocument/foldingRange" => Folding(syntax),
            "textDocument/selectionRange" => Selections(syntax, parameters, token),
            _ => throw Invalid("Unknown syntax feature.", -32601)
        };
    }

    private static object[] DocumentSymbols(SyntaxStructure syntax, bool hierarchical, CancellationToken token)
    {
        if (syntax.Declarations.Count > 1000) throw Invalid("Document outline exceeds 1,000 declarations.", -32803);
        var declarations = syntax.Declarations.Where(item => item.Kind != "parameter").ToArray();
        if (!hierarchical) return declarations.Select(item => (object)new
        {
            name = item.Name, kind = SymbolKind(item.Kind),
            location = new { uri = FileUris.Uri(item.Span.Path), range = FeatureProjection.Range(syntax.Document, item.Span) }
        }).ToArray();
        var children = declarations.ToLookup(item => item.Owner);
        object Node(SyntaxDeclaration item, int depth)
        {
            token.ThrowIfCancellationRequested();
            if (depth > 32) throw Invalid("Document outline nesting exceeds 32 levels.", -32803);
            return new { name = item.Name, kind = SymbolKind(item.Kind), range = FeatureProjection.Range(syntax.Document, item.Span),
                selectionRange = FeatureProjection.Token(item.NameSpan), children = children[item.NameSpan.Offset].Select(child => Node(child, depth + 1)).ToArray() };
        }
        var offsets = declarations.Select(item => item.NameSpan.Offset).ToHashSet();
        return declarations.Where(item => item.Owner is null || !offsets.Contains(item.Owner.Value)).Select(item => Node(item, 0)).ToArray();
    }

    private static object Folding(SyntaxStructure syntax)
    {
        if (syntax.Folds.Count > 1000) throw Invalid("Document exceeds 1,000 folding regions.", -32803);
        return syntax.Folds.Select(span => new
        {
            startLine = span.Line - 1,
            endLine = syntax.Document.Span(span.Offset + span.Length - 1, 0).Line - 1
        }).Where(fold => fold.endLine > fold.startLine).Distinct().ToArray();
    }

    private static object Selections(SyntaxStructure syntax, JObject parameters, CancellationToken token)
    {
        if (parameters["positions"] is not JArray positions || positions.Count > 100)
            throw Invalid("Supply at most 100 selection positions.");
        return positions.Select(value =>
        {
            token.ThrowIfCancellationRequested();
            var point = Object(value, "position");
            var offset = syntax.Document.Offset(checked(RequiredInteger(point, "line") + 1), checked(RequiredInteger(point, "character") + 1));
            var chain = new List<SourceSpan> { syntax.Document.Span(offset, 0) };
            foreach (var span in syntax.Selections.Where(item => item.Contains(offset)).OrderBy(item => item.Length))
            {
                var inner = chain[^1];
                if (span.Offset <= inner.Offset && span.Offset + span.Length >= inner.Offset + inner.Length && span.Length > inner.Length) chain.Add(span);
                if (chain.Count >= 32) break;
            }
            object? parent = null;
            for (var index = chain.Count - 1; index >= 0; index--)
                parent = new { range = FeatureProjection.Range(syntax.Document, chain[index]), parent };
            return parent;
        }).ToArray();
    }

    private static int SymbolKind(string kind) => kind switch
    {
        "container" => 3, "record" => 23, "interface" => 11, "union" or "domain" => 10,
        "routine" => 12, "property" => 7, "union_case" or "enum_member" => 22, "constant" => 14, _ => 13
    };
}
