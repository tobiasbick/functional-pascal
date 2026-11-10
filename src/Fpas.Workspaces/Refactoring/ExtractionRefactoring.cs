using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> ExtractVariable(Draft draft, RefactoringRequest request, SourceDocument source, int start, int end)
    {
        var name = IdentifierFacts.Spell(request.Name ?? "");
        while (start < end && char.IsWhiteSpace(source.Text[start])) start++;
        while (end > start && char.IsWhiteSpace(source.Text[end - 1])) end--;
        var extraction = start < end ? draft.Syntax(source).Extract(start, end) : null;
        if (extraction is null)
            throw Refuse("Select a complete value expression that is evaluated first in its statement and stays in the same block.");

        SemanticType? type = null;
        string? spelling = null;
        var imports = draft.Syntax(source).Imports()?.Items ?? [];
        foreach (var (_, snapshot) in draft.Owners(source.Path))
        {
            var selected = ExactType(snapshot, extraction.Expression);
            if (selected is null || selected.Kind is "void" or "null" or "byref" or "unresolved" || selected.Callable is not null)
                throw Refuse("The selected expression has no explicit, storable FPAS type.");
            if (type is not null && !SameType(type, selected))
                throw Refuse("The selected expression has a different type in a linked owner.");
            if (extraction.PriorCallTargets.Any(offset => !snapshot.Operations.Any(operation => operation.Kind == "invoke" &&
                operation.Anchor is { } anchor && Paths.Equals(anchor.Path, source.Path) && anchor.Offset == offset)))
                throw Refuse("Extraction would move evaluation before an indirect call receiver.");
            type = selected;
            spelling ??= TypeSpelling(snapshot, selected, source.Path, imports);
        }
        if (spelling is null) throw Refuse("No checked source owner is available.");

        var line = source.Span(extraction.Statement, 0).Line;
        var indent = source.Text[source.Offset(line, 1)..extraction.Statement];
        var newline = LineEnding(source.Text, extraction.Statement);
        var separator = indent.All(character => character is ' ' or '\t') ? newline + indent : " ";
        var prefix = "const " + name + ": " + spelling + " := ";
        draft.Edit(source, extraction.Statement, extraction.Statement, prefix + source.Text[start..end] + ";" + separator);
        draft.Edit(source, start, end, name);
        draft.Move = (extraction.Expression, extraction.Statement, prefix.Length);
        draft.Extracted = (extraction.Expression, extraction.Statement, request.Name!);
        return ["The expression remains in the original statement's block and is evaluated once before that statement."];
    }

    private static SemanticType? ExactType(SemanticSnapshot snapshot, SourceSpan span)
    {
        var expression = snapshot.Expressions.FirstOrDefault(item => Paths.Equals(item.Span.Path, span.Path) &&
            item.Span.Offset == span.Offset && item.Span.Length == span.Length);
        return expression is null ? null : snapshot.Types.Single(type => type.Id == expression.Type);
    }

    private static bool SameType(SemanticType left, SemanticType right)
        => left.Kind == right.Kind && left.Display == right.Display && left.Nullable == right.Nullable &&
            left.Metadata?.GetKey(includeAssemblyHash: false) == right.Metadata?.GetKey(includeAssemblyHash: false);

    private static string TypeSpelling(SemanticSnapshot snapshot, SemanticType type, string path, IReadOnlyList<RefactoringImport> imports)
    {
        SemanticType Find(string id) => snapshot.Types.Single(candidate => candidate.Id == id);
        var nullable = type.Nullable ? "?" : "";
        if (type.Kind == "array" && type.Element is { } element)
        {
            var rank = type.Rank == 1 ? "" : "[" + new string(',', type.Rank - 1) + "]";
            var array = $"array{rank} of {Spell(element)}";
            return type.Nullable ? $"({array})?" : array;
        }
        var name = type.Display.TrimEnd('?').Split('<')[0];
        if (name.Length == 0 || name[0] is '!' or '<') throw Refuse("The selected value has no expressible FPAS type.");
        var alias = imports.Where(item => name == item.TargetName || name.StartsWith(item.TargetName + ".", StringComparison.Ordinal) ||
            name.StartsWith(item.TargetName + "+", StringComparison.Ordinal) || name.StartsWith(item.TargetName + "`", StringComparison.Ordinal))
            .OrderByDescending(item => item.TargetName.Length).FirstOrDefault();
        if (alias is not null) name = IdentifierFacts.Spell(alias.AliasName) + name[alias.TargetName.Length..];
        else if (snapshot.Symbols.FirstOrDefault(symbol => symbol.Context?.QualifiedName == name &&
            symbol.Declaration is { } declaration && Paths.Equals(declaration.Path, path) &&
            symbol.Kind is "record" or "interface" or "variant" or "domain" or "enum") is { } own)
            name = IdentifierFacts.Spell(own.Name);
        var args = type.Arguments.Select(Spell).ToArray();
        if (!name.Contains('`')) return name.Replace('+', '.') + Arguments(args) + nullable;
        var parts = name.Split('+');
        var consumed = 0;
        for (var index = 0; index < parts.Length; index++)
        {
            var part = parts[index];
            var marker = part.IndexOf('`');
            if (marker < 0) continue;
            if (!int.TryParse(part.AsSpan(marker + 1), out var count) || count < 1 || consumed + count > args.Length)
                throw Refuse("The selected generic value has no expressible FPAS type.");
            parts[index] = part[..marker] + Arguments(args[consumed..(consumed + count)]);
            consumed += count;
        }
        if (consumed != args.Length) throw Refuse("The selected nested generic type cannot be spelled in this scope.");
        return string.Join('.', parts) + nullable;

        string Spell(string id) => TypeSpelling(snapshot, Find(id), path, imports);
        static string Arguments(string[] values) => values.Length == 0 ? "" : "<" + string.Join(", ", values) + ">";
    }

    private static string LineEnding(string text, int start)
    {
        var next = text.IndexOfAny(['\r', '\n'], start);
        if (next < 0) next = text.IndexOfAny(['\r', '\n']);
        return next < 0 ? "\n" : text[next] == '\n' ? "\n" : next + 1 < text.Length && text[next + 1] == '\n' ? "\r\n" : "\r";
    }
}
