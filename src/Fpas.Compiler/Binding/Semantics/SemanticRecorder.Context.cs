using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private Dictionary<string, Dictionary<int, SymbolEntry>>? declarationSymbols;
    private readonly Dictionary<string, ILookup<int?, DeclarationSyntax>> declarationChildren = new(StringComparer.Ordinal);

    private DeclarationContext? Context(SymbolEntry entry)
    {
        if (entry.Declaration is not { } name || !sourceDeclarations[name.Path].TryGetValue(name.Offset, out var declaration)) return null;
        var syntax = sourceDeclarations[name.Path];
        var document = documents[name.Path];
        declarationSymbols ??= symbols.Values.Distinct().Where(symbol => symbol.Declaration is not null)
            .GroupBy(symbol => symbol.Declaration!.Path).ToDictionary(group => group.Key, group => group.ToDictionary(symbol => symbol.Declaration!.Offset));
        var byPosition = declarationSymbols[name.Path];
        if (!declarationChildren.TryGetValue(name.Path, out var children)) declarationChildren.Add(name.Path, children = syntax.Values.ToLookup(child => child.Owner));
        var owner = declaration.Owner is { } parent ? byPosition.GetValueOrDefault(parent)?.Id : null;
        var names = new Stack<string>();
        for (var current = declaration; ;)
        {
            token.ThrowIfCancellationRequested(); names.Push(current.SourceName);
            if (current.Owner is not { } position) break;
            current = syntax[position];
        }
        var roots = new HashSet<string>(StringComparer.Ordinal);
        var referenced = new HashSet<string>(StringComparer.Ordinal);
        Add(declaration);
        if (entry.Identity is ScalarSymbol scalar) roots.Add(Type(scalar.Base));
        if (entry.Type is not null) roots.Add(entry.Type);
        var pending = new Stack<int>(); pending.Push(name.Offset);
        while (pending.TryPop(out var position))
            foreach (var child in children[position])
            {
                token.ThrowIfCancellationRequested();
                if (!byPosition.TryGetValue(child.Name.Position, out var symbol) || symbol.Kind == "local") continue;
                if (position != name.Offset && byPosition[position].Kind == "routine" && symbol.Kind is not ("parameter" or "type_parameter")) continue;
                if (position == name.Offset && entry.Kind is not ("container" or "record" or "interface" or "union" or "union_case" or "domain") && symbol.Kind is not ("parameter" or "type_parameter")) continue;
                Add(child);
                if (symbol.Type is not null) roots.Add(symbol.Type);
                if (symbol.Kind is "routine" or "union_case") pending.Push(child.Name.Position);
            }
        var doc = declaration.Documentation;
        return new(owner, string.Join('.', names), declaration.Accessibility, document.Span(declaration.Start, declaration.End - declaration.Start),
            declaration.BodyStart is { } body ? document.Span(body, declaration.BodyEnd!.Value - body) : null,
            doc is { Lines.Count: > 0 } ? document.Span(doc.Lines[0].Start, doc.Lines[^1].End - doc.Lines[0].Start) : null,
            Array.AsReadOnly(roots.Order(StringComparer.Ordinal).ToArray()), Array.AsReadOnly(referenced.Order(StringComparer.Ordinal).ToArray()));

        void Add(DeclarationSyntax item)
        {
            foreach (var type in item.Types)
            {
                if (surfaceTypes.TryGetValue((name.Path, type.Position, type.SpanEnd), out var id)) roots.Add(id);
                foreach (var ((path, offset, role), occurrence) in occurrences)
                {
                    token.ThrowIfCancellationRequested();
                    if (path == name.Path && role == "type" && offset >= type.Position && offset < type.SpanEnd && occurrence.Symbol.Type is not null)
                        referenced.Add(occurrence.Symbol.Id);
                }
            }
        }
    }
}
