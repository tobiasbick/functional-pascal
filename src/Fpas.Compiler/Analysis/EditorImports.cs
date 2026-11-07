using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

public sealed record EditorImport(SourceSpan Receiver, SourceSpan Member, string Alias, string QualifiedName, string? AssemblyHash,
    DiagnosticEdit Edit);

public static class EditorImports
{
    public static EditorImport? Suggest(AnalysisRequest request, string path, int offset, CancellationToken token = default)
    {
        var paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        var input = request.Sources.Single(item => paths.Equals(item.Path, path));
        var source = new SourceText(path, input.Text, token);
        var tokens = new Lexer(source).ReadTokens();
        var index = Array.FindIndex(tokens.ToArray(), item => item.Position == offset && item.Kind == TokenKind.Identifier);
        if (index < 0 || index + 2 >= tokens.Count || tokens[index + 1].Text != "." || tokens[index + 2].Kind != TokenKind.Identifier ||
            index > 0 && tokens[index - 1].Text == ".") return null;
        var receiver = tokens[index]; var member = tokens[index + 2];
        // A failed body may have no binding facts even for tokens before its error.
        // Introducing this alias must not change how any other occurrence is resolved.
        if (tokens.Count(item => item.Kind == TokenKind.Identifier && item.Text == receiver.Text) != 1) return null;
        try
        {
            var unit = new Parser(source, tokens, recover: true).ParseCompilationUnit();
            if (unit.Imports.Any(item => item.Alias.Text == receiver.Text)) return null;
            using var references = ReferenceCatalog.Open(request.References, source, request.ReferenceImages, captureFingerprints: true);
            var candidates = references.EditorImportTypes(receiver.Text, token).Select(type =>
                (Qualified: type.FullName, Hash: type.Assembly.ContentHash)).Take(2).ToList();
            foreach (var candidate in request.Sources.Where(item => !paths.Equals(item.Path, path)))
            {
                if (candidates.Count > 1) return null;
                var other = new SourceText(candidate.Path, candidate.Text, token);
                var declared = new Parser(other, new Lexer(other).ReadTokens(), recover: true).ParseCompilationUnit();
                if (!declared.IsProgram && declared.Name.Parts[^1].Text == receiver.Text)
                    candidates.Add((declared.Name.Text, null));
            }
            if (candidates.Count != 1) return null;
            var target = candidates[0];
            if (unit.Imports.Any(item => item.Target.Text == target.Qualified)) return null;
            var alias = IdentifierFacts.Spell(receiver.Text);
            var qualified = string.Join('.', target.Qualified.Split('.').Select(IdentifierFacts.Spell));
            var newline = input.Text.Contains("\r\n", StringComparison.Ordinal) ? "\r\n" : "\n";
            var at = tokens.First(item => item.Text == ";").Position + 1;
            var text = newline + "uses " + qualified + " as " + alias + ";";
            if (unit.Imports.LastOrDefault() is { } last)
            {
                at = last.Alias.Position + last.Alias.Length;
                text = ", " + qualified + " as " + alias;
            }
            var document = new SourceDocument(path, input.Text, token);
            return new(document.Span(receiver.Position, receiver.Length), document.Span(member.Position, member.Length),
                receiver.Text, target.Qualified, target.Hash, new(source.Location(at, 0), "", text));
        }
        catch (CompilationFailure) { return null; }
    }
}
