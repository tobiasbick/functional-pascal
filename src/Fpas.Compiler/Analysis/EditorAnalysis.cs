using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Analysis;

public sealed record EditorCandidate(string Name, string Kind, string Signature, MetadataIdentity? Metadata = null, SourceSpan? Declaration = null);
public sealed record EditorCompletion(string Context, SourceSpan Replacement, IReadOnlyList<EditorCandidate> Items, bool Incomplete);
public sealed record EditorCallSignature(string Label, IReadOnlyList<string> Parameters, int? ActiveParameter, MetadataIdentity? Metadata = null);
public sealed record EditorCallSignatures(IReadOnlyList<EditorCallSignature> Items, int? ActiveSignature, bool Incomplete);

/// <summary>Checks a cursor against captured inputs. Any recovery edits exist only for the duration of the query.</summary>
public static class EditorAnalysis
{
    internal const string Marker = "__fpas_editor_cursor";
    private static readonly StringComparer Paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    public static EditorCompletion Complete(AnalysisRequest request, string path, int offset, CancellationToken token = default)
    {
        var input = request.Sources.Single(item => Paths.Equals(item.Path, path));
        var document = new SourceDocument(path, input.Text, token);
        var tokens = Lex(input, token, out var comments);
        var name = tokens.FirstOrDefault(item => item.Kind == TokenKind.Identifier && item.Position <= offset && offset <= item.Position + item.Length);
        var start = name?.Position ?? offset;
        var span = document.Span(start, name?.Length ?? 0);
        if (comments.Any(item => item.Start <= offset && offset < item.End) || tokens.Any(item =>
            item.Kind is TokenKind.String or TokenKind.Character && item.Position < offset && offset < item.Position + item.Length))
            return new("none", span, [], false);
        var prefix = input.Text[start..offset].TrimStart('@');
        var previous = tokens.TakeWhile(item => item.Position < start).ToArray();
        var before = previous.LastOrDefault()?.Text;
        var statement = previous.Skip(Array.FindLastIndex(previous, item => item.Text is ";" or "begin" or "then" or "do" or "else") + 1).ToArray();
        var import = statement.FirstOrDefault()?.Text == "uses";
        var type = before is ":" or "new" || before == "(" && previous.Length >= 2 && previous[^2].Text is "cast" or "typeof";
        var kind = import ? "import" : type ? "type" : before == "." ? "member" : "expression";
        if (before is "program" or "unit" or "function" or "procedure" or "var" or "const" or "type" or "as" ||
            import && statement.Skip(Array.FindLastIndex(statement, item => item.Text == ",") + 1).Any(item => item.Text == "as"))
            return new("declaration", span, [], false);
        var probe = new EditorProbe(kind, prefix, start);
        try
        {
            if (import || type)
            {
                using var prepared = PreparedCompilation.Create(request, token, collectSemantics: true);
                var owner = prepared.Catalog.Units.FirstOrDefault(item => Paths.Equals(item.Source.Path, path));
                if (import)
                {
                    var pathTokens = statement.Skip(Array.FindLastIndex(statement, item => item.Text is "uses" or ",") + 1);
                    var qualifier = string.Concat(pathTokens.Select(item => item.Text));
                    foreach (var candidate in prepared.References.EditorPaths(qualifier, prefix, token)) probe.Add(candidate);
                    foreach (var unit in prepared.Catalog.Units.Where(unit => unit != owner && unit.Syntax.Name.Text.StartsWith(qualifier, StringComparison.Ordinal)))
                        probe.Add(new(unit.Syntax.Name.Text[qualifier.Length..], "namespace", unit.Syntax.Name.Text));
                }
                else
                {
                    foreach (var primitive in new[] { "bool", "byte", "sbyte", "short", "ushort", "int", "uint", "long", "ulong", "nint",
                        "nuint", "float", "double", "decimal", "char", "string", "object" })
                        probe.Add(new(primitive, "type", primitive));
                    if (owner is not null)
                    {
                        foreach (var alias in owner.Syntax.Imports)
                            probe.Add(new(alias.Alias.Text, "type", alias.Target.Text));
                        foreach (var candidate in owner.Records.Keys.Concat(owner.Interfaces.Keys).Concat(owner.Unions.Keys).Concat(owner.Scalars.Keys).Concat(owner.Aliases.Keys))
                            probe.Add(new(candidate, "type", candidate));
                    }
                }
            }
            else
            {
                var text = input.Text.Remove(start, span.Length).Insert(start, Marker);
                var lineStart = start == 0 ? 0 : input.Text.LastIndexOfAny(['\r', '\n'], start - 1) + 1;
                var lead = input.Text[lineStart..start].Trim();
                var after = start + Marker.Length;
                var next = tokens.FirstOrDefault(item => item.Position >= start + span.Length && !ReferenceEquals(item, name));
                if (next is null || next.Kind == TokenKind.End || next.Text == "end") text = text.Insert(after, ";");
                if (lead.Length == 0 || before == "." && lead.All(character => char.IsLetterOrDigit(character) || character is '@' or '_' or '.'))
                {
                    var indent = lineStart;
                    while (indent < start && char.IsWhiteSpace(input.Text[indent])) indent++;
                    text = text.Insert(indent, "discard ");
                    probe.Position += "discard ".Length;
                }
                Run(request, input, text, probe, token);
            }
        }
        catch (CompilationFailure) { /* The cursor may lie outside a recoverable compilation. */ }
        return new(kind, span, probe.Items, probe.Incomplete || !probe.Reached);
    }

    public static EditorCallSignatures Signatures(AnalysisRequest request, string path, int offset, CancellationToken token = default)
    {
        var input = request.Sources.Single(item => Paths.Equals(item.Path, path));
        var tokens = Lex(input, token, out var comments);
        if (comments.Any(item => item.Start <= offset && offset < item.End)) return new([], null, false);
        var stack = new Stack<(int Index, int Argument, int Start)>();
        for (var index = 0; index < tokens.Count && tokens[index].Position < offset; index++)
        {
            var item = tokens[index];
            if (item.Kind != TokenKind.Symbol) continue;
            if (item.Text is "(" or "[") stack.Push((index, 0, index + 1));
            else if (item.Text is ")" or "]" && stack.Count > 0) stack.Pop();
            else if (item.Text is ";" && stack.Count > 0) stack.Clear();
            else if (item.Text == "," && stack.TryPop(out var current)) stack.Push((current.Index, current.Argument + 1, index + 1));
        }
        if (!stack.TryPeek(out var call) || tokens[call.Index].Text != "(") return new([], null, false);
        var text = input.Text;
        var probe = new EditorProbe("signature", "", tokens[call.Index].Position) { Slot = call.Argument, Cursor = offset };
        var tail = call.Start < tokens.Count ? tokens[call.Start] : null;
        if (tail?.Kind == TokenKind.Identifier && call.Start + 1 < tokens.Count && tokens[call.Start + 1].Text == ":=" &&
            tokens[call.Start + 1].Position + 2 <= offset)
            probe.ArgumentName = tail.Text;
        var depth = 1;
        var boundary = tokens.Skip(call.Index + 1).First(item =>
        {
            if (item.Kind == TokenKind.Symbol && item.Text == "(") depth++;
            if (item.Kind == TokenKind.Symbol && item.Text == ")") depth--;
            return depth == 0 || item.Kind == TokenKind.End || item.Text is ";" or "end";
        });
        if (depth > 0)
        {
            if (depth > 16) return new([], null, true);
            text = text.Insert(boundary.Position, new string(')', depth) + (boundary.Text == ";" ? "" : ";"));
            probe.Incomplete = true;
        }
        var trailing = tokens.LastOrDefault(item => item.Position < boundary.Position);
        if (trailing is { Kind: TokenKind.Symbol, Text: "," or ":=" })
        {
            text = trailing.Text == "," ? text.Remove(trailing.Position, trailing.Length) : text.Insert(trailing.Position + trailing.Length, " 0");
            probe.Incomplete = true;
        }
        try { Run(request, input, text, probe, token); }
        catch (CompilationFailure) { /* An unavailable receiver supplies no signature. */ }
        return new(probe.Signatures, probe.ActiveSignature, probe.Incomplete);
    }

    private static IReadOnlyList<Token> Lex(CompilationSource input, CancellationToken token, out List<SourceComment> comments)
    {
        comments = [];
        return new Lexer(new(input.Path, input.Text, token), comments).ReadTokens();
    }

    private static void Run(AnalysisRequest request, CompilationSource input, string text, EditorProbe probe, CancellationToken token)
    {
        using var prepared = PreparedCompilation.Create(request with
        {
            Sources = request.Sources.Select(item => item == input ? item with { Text = text } : item).ToArray(),
            ValidateDocumentation = false
        }, token, collectSemantics: true, editor: (input.Path, probe));
    }
}

internal sealed class EditorProbe(string context, string prefix, int position)
{
    private readonly Dictionary<string, EditorCandidate> candidates = new(StringComparer.Ordinal);
    internal string Context { get; } = context;
    internal int Position { get; set; } = position;
    internal int Slot { get; init; }
    internal int Cursor { get; init; }
    internal string? ArgumentName { get; set; }
    internal bool Reached { get; set; }
    internal bool Incomplete { get; set; }
    internal IReadOnlyList<EditorCandidate> Items => candidates.Values.OrderBy(item => item.Name, StringComparer.Ordinal).ToArray();
    internal IReadOnlyList<EditorCallSignature> Signatures { get; set; } = [];
    internal int? ActiveSignature { get; set; }
    internal bool Matches(Token name) => name.Position == Position && name.Text == EditorAnalysis.Marker;
    internal bool Accepts(string name) => name.StartsWith(prefix, StringComparison.OrdinalIgnoreCase);

    internal void Add(EditorCandidate candidate)
    {
        Reached = true;
        if (!Accepts(candidate.Name) || candidates.ContainsKey(candidate.Name)) return;
        if (candidates.Count >= 200) { Incomplete = true; return; }
        candidates.Add(candidate.Name, candidate);
    }
}
