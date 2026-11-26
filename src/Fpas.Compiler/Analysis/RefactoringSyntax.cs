// [Documentation](../../../docs/pascal/tools/refactoring.md)
using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

/// <summary>Parameter extents of an editable routine.</summary>
/// <param name="Insertion">The offset where a new required parameter is inserted: before the first optional parameter, or at the closing parenthesis.</param>
/// <param name="FirstOptional">The index of the first parameter with a default, or the parameter count.</param>
/// <param name="Parameters">Parameters in declaration order.</param>
/// <param name="HasVariadicParameter">Whether the last parameter is a params parameter.</param>
/// <param name="ParameterList">The extent inside the parameter list's parentheses, including trivia.</param>
public sealed record RefactoringRoutine(int Insertion, int FirstOptional, IReadOnlyList<RefactoringListItem> Parameters,
    bool HasVariadicParameter, SourceSpan ParameterList);

/// <summary>Argument extents of one call.</summary>
/// <param name="Close">The offset of the closing parenthesis.</param>
/// <param name="Arguments">Arguments in written evaluation order.</param>
public sealed record RefactoringCall(int Close, IReadOnlyList<RefactoringArgument> Arguments);

/// <summary>One parameter or argument in its original source list.</summary>
/// <param name="Name">Declared parameter name or supplied argument label, null for a positional argument.</param>
/// <param name="Span">Exact source extent including the label or mode.</param>
public record RefactoringListItem(string? Name, SourceSpan Span);

/// <summary>Identifies the parts of a call argument that can be edited independently.</summary>
/// <param name="Name">The argument label, or null for a positional argument.</param>
/// <param name="Span">The entire written argument.</param>
/// <param name="Value">The argument expression without its label or mode.</param>
/// <param name="Modifier">The exact mode token, or null for a value argument.</param>
/// <param name="Storage">The name token of a simple storage expression, when it has one.</param>
public sealed record RefactoringArgument(string? Name, SourceSpan Span, SourceSpan Value, SourceSpan? Modifier, int? Storage)
    : RefactoringListItem(Name, Span);

/// <summary>Identifies the editable parts of a routine parameter.</summary>
/// <param name="Owner">The routine name offset.</param>
/// <param name="Index">The parameter slot in declaration order.</param>
/// <param name="Mode">The written mode, or the empty string for a value parameter.</param>
/// <param name="Modifier">The exact mode token, if present.</param>
/// <param name="Type">The complete parameter type annotation.</param>
public sealed record RefactoringParameter(int Owner, int Index, string Mode, SourceSpan? Modifier, SourceSpan Type);

/// <summary>Extents of one uses statement.</summary>
/// <param name="Start">The offset of the uses keyword.</param>
/// <param name="Target">The offset of the last part of the imported name.</param>
/// <param name="Alias">The offset of the alias.</param>
/// <param name="End">The end offset of the terminating semicolon.</param>
/// <param name="TargetName">The qualified imported name.</param>
/// <param name="AliasName">The alias.</param>
public sealed record RefactoringImport(int Start, int Target, int Alias, int End, string TargetName, string AliasName);

/// <summary>Extents of the uses statements in a source.</summary>
/// <param name="Start">The offset of the first uses keyword.</param>
/// <param name="End">The end offset of the last import's semicolon.</param>
/// <param name="Items">The imports in source order.</param>
/// <param name="Comments">All comment ranges of the source, so edits can preserve them.</param>
public sealed record RefactoringImports(int Start, int End, IReadOnlyList<RefactoringImport> Items, IReadOnlyList<SourceSpan> Comments);

/// <summary>An expression that is evaluated first by its statement and can move into a preceding constant.</summary>
/// <param name="Expression">The selected expression.</param>
/// <param name="Statement">The offset of the containing statement, where the constant is inserted.</param>
/// <param name="PriorCallTargets">The name offsets of enclosing calls whose first argument contains the expression.</param>
public sealed record RefactoringExtraction(SourceSpan Expression, int Statement, IReadOnlyList<int> PriorCallTargets);

/// <summary>Insertion offsets for adding a protocol and its members to a record.</summary>
/// <param name="HeaderEnd">The end of the record keyword or the last implemented interface.</param>
/// <param name="BodyEnd">The start of the closing end record tokens.</param>
/// <param name="HasInterfaces">Whether the header already has an implements clause.</param>
public sealed record RefactoringRecord(int HeaderEnd, int BodyEnd, bool HasInterfaces);

/// <summary>Source extents for edits that must follow FPAS grammar and evaluation order.</summary>
public sealed partial class RefactoringSyntax
{
    private readonly SourceDocument document;
    private readonly SourceText source;
    private readonly IReadOnlyList<Token> tokens;
    private readonly List<SourceComment> comments = [];
    private readonly CompilationUnitSyntax unit;
    private readonly CancellationToken token;

    /// <summary>Parses a syntactically valid source.</summary>
    /// <param name="document">The source to parse.</param>
    /// <param name="token">Cancels parsing and later queries with an <see cref="OperationCanceledException"/>.</param>
    public RefactoringSyntax(SourceDocument document, CancellationToken token)
    {
        this.document = document; this.token = token;
        source = new SourceText(document.Path, document.Text, token);
        tokens = new Lexer(source, comments).ReadTokens();
        unit = new Parser(source, tokens).ParseCompilationUnit();
    }

    /// <summary>Finds insertion points for an editable record named at the given offset.</summary>
    /// <param name="offset">Start of the record's name token.</param>
    /// <returns>The original header and body offsets, or null for another declaration.</returns>
    public RefactoringRecord? Record(int offset)
    {
        var record = unit.Records.SingleOrDefault(item => item.Name.Position == offset);
        if (record is null) return null;
        var declaration = unit.Declarations[offset];
        var lastInterface = record.Interfaces?.LastOrDefault();
        var headerEnd = lastInterface?.SpanEnd ?? End(tokens.Last(item => item.Position > offset &&
            item.Position < declaration.BodyStart && item.Kind == TokenKind.Keyword && item.Text == "record"));
        return new(headerEnd, declaration.BodyEnd!.Value, lastInterface is not null);
    }

    /// <summary>Gets the offset after the program or unit header's semicolon.</summary>
    public int ContainerHeaderEnd => End(tokens[FindIndex(item => item.Position == unit.Name.Parts[^1].Position) + 1]);

    /// <summary>Finds the parameter extents of a source routine without attributes.</summary>
    /// <param name="offset">The offset of the routine name.</param>
    /// <returns>The extents, or <see langword="null"/> when no such routine is declared there.</returns>
    public RefactoringRoutine? EditableRoutine(int offset)
    {
        var routine = FindRoutine(offset);
        if (routine is null) return null;
        var open = FindIndex(item => item.Position >= End(routine.TypeParameters?.LastOrDefault() ?? routine.Name) && Symbol(item, "("));
        var close = Close(open);
        if (close < 0) return null;
        var optional = routine.Parameters.TakeWhile(parameter => parameter.Default is null).Count();
        var insertion = optional < routine.Parameters.Count
            ? unit.Declarations[routine.Parameters[optional].Name.Position].Start
            : tokens[close].Position;
        return new(insertion, optional, routine.Parameters.Select(parameter =>
        {
            var declaration = unit.Declarations[parameter.Name.Position];
            return new RefactoringListItem(parameter.Name.Text, document.Span(declaration.Start, declaration.End - declaration.Start));
        }).ToArray(),
            routine.Parameters.LastOrDefault()?.Mode == "params",
            document.Span(End(tokens[open]), tokens[close].Position - End(tokens[open])));
    }

    /// <summary>Finds the declaring routine of an ordinary value parameter selected by name.</summary>
    /// <param name="offset">Start of the parameter name token.</param>
    /// <returns>The routine name offset, or null for other declarations and parameter modes.</returns>
    public int? ValueParameterOwner(int offset) =>
        unit.Declarations.TryGetValue(offset, out var declaration) && declaration.Kind == "parameter" && declaration.Start == offset
            ? declaration.Owner : null;

    /// <summary>Finds one parameter by name in a source routine without attributes or variadic parameters.</summary>
    /// <param name="offset">Start of the parameter name token.</param>
    /// <returns>The editable parameter parts, or null when the declaration is outside that scope.</returns>
    public RefactoringParameter? EditableParameter(int offset)
    {
        if (!unit.Declarations.TryGetValue(offset, out var declaration) || declaration.Kind != "parameter" ||
            declaration.Owner is not int owner || FindRoutine(owner) is not { } routine ||
            routine.Parameters.LastOrDefault()?.Mode == "params") return null;
        for (var index = 0; index < routine.Parameters.Count; index++)
        {
            var parameter = routine.Parameters[index];
            if (parameter.Name.Position != offset) continue;
            var type = parameter.Type;
            return new(owner, index, parameter.Mode,
                parameter.Mode.Length == 0 ? null : document.Span(declaration.Start, parameter.Mode.Length),
                document.Span(type.Position, type.SpanEnd - type.Position));
        }
        return null;
    }

    private RoutineSyntax? FindRoutine(int offset)
        => unit.Routines.Concat(unit.Records.SelectMany(record => record.Methods))
            .SingleOrDefault(item => item.Name.Position == offset && item.Attributes is not { Count: > 0 });

    /// <summary>Selects one item and an adjacent separator for deletion without erasing comments.</summary>
    /// <param name="items">Parameters or arguments in source order.</param>
    /// <param name="index">Index to delete.</param>
    /// <returns>The deletion range, or null when the range contains a comment.</returns>
    public SourceSpan? RemovalRange(IReadOnlyList<RefactoringListItem> items, int index)
    {
        var span = items[index].Span;
        var start = span.Offset;
        var end = start + span.Length;
        if (index + 1 < items.Count) end = items[index + 1].Span.Offset;
        else if (index > 0) start = items[index - 1].Span.Offset + items[index - 1].Span.Length;
        var range = document.Span(start, end - start);
        return ContainsComment(range) ? null : range;
    }

    /// <summary>Checks whether replacing the given source span would erase a comment.</summary>
    /// <param name="span">The range within this source document.</param>
    /// <returns>True when a comment overlaps the range.</returns>
    public bool ContainsComment(SourceSpan span)
        => comments.Any(comment => comment.Start < span.Offset + span.Length && span.Offset < comment.End);

    /// <summary>Finds the argument extents of a call expression.</summary>
    /// <param name="span">The whole call expression, ending at its closing parenthesis.</param>
    /// <param name="target">An offset of the called name; it must precede the closing parenthesis.</param>
    /// <returns>The extents, or <see langword="null"/> when <paramref name="span"/> is not exactly a call.</returns>
    public RefactoringCall? Call(SourceSpan span, int target)
    {
        var first = FindIndex(item => item.Position == span.Offset);
        var close = FindIndex(item => End(item) == span.Offset + span.Length && Symbol(item, ")"));
        if (first < 0 || close < first || target >= tokens[close].Position) return null;
        var fragment = tokens.Skip(first).Take(close - first + 1).Append(tokens[^1]).ToArray();
        var parsed = new Parser(source, fragment).ParseExpressionFragment();
        return parsed is CallSyntax call && call.SpanEnd == span.Offset + span.Length
            ? new(tokens[close].Position, call.Arguments.Select(Argument).ToArray())
            : null;

        RefactoringArgument Argument(ExpressionSyntax expression)
        {
            var named = expression as NamedArgumentSyntax;
            var value = named?.Value ?? expression;
            var modifier = value as ArgumentModeSyntax;
            value = modifier?.Value ?? value;
            var storage = value switch
            {
                NameExpressionSyntax name => name.Name.Position,
                MemberSyntax member => member.Name.Position,
                _ => (int?)null
            };
            return new(named?.Name.Text, Extent(expression), Extent(value),
                modifier is null ? null : document.Span(modifier.Position, modifier.Mode.Length), storage);
        }

        SourceSpan Extent(ExpressionSyntax expression)
            => document.Span(expression.SpanStart, expression.SpanEnd - expression.SpanStart);
    }

    /// <summary>Finds the extents of the uses statements.</summary>
    /// <returns>The extents, or <see langword="null"/> when the source imports nothing.</returns>
    public RefactoringImports? Imports()
    {
        if (unit.Imports.Count == 0) return null;
        var imports = unit.Imports.Select(import =>
        {
            var first = FindIndex(item => item.Position == import.Target.Position);
            var last = FindIndex(item => item.Position == import.Alias.Position);
            return new RefactoringImport(tokens[first - 1].Position, import.Target.Parts[^1].Position,
                import.Alias.Position, End(tokens[last + 1]), import.Target.Text, import.Alias.Text);
        }).ToArray();
        return new(imports[0].Start, imports[^1].End, imports,
            comments.Select(comment => document.Span(comment.Start, comment.End - comment.Start)).ToArray());
    }

    /// <summary>Finds a selected expression that its statement evaluates first and that can be extracted.</summary>
    /// <param name="start">The selection start offset.</param>
    /// <param name="end">The selection end offset.</param>
    /// <returns>The extraction, or <see langword="null"/> when the selection cannot move before its statement.</returns>
    public RefactoringExtraction? Extract(int start, int end)
    {
        if (tokens.Any(item => item.Position >= start && item.Position < end && item.Kind is not (TokenKind.String or TokenKind.Character) &&
            item.Text is "await" or "ref" or "out" or "in" or "function" or "procedure"))
            return null;
        foreach (var statements in StatementLists())
        {
            foreach (var statement in statements)
            {
                token.ThrowIfCancellationRequested();
                var anchor = statement is BindingSyntax binding ? unit.Declarations[binding.Name.Position].Start : statement.Position;
                List<(int Offset, bool RequiresStatic)> calls = [];
                if (FirstExpression(statement) is { } expression && EvaluatedFirst(expression, start, end, calls, allowQualified: false))
                    return new(document.Span(start, end - start), anchor, calls.Select(call => call.Offset).ToArray());
            }
        }
        return null;
    }

    private IEnumerable<IReadOnlyList<StatementSyntax>> StatementLists()
        => StatementBlocks().Select(block => block.Statements);

    private IEnumerable<(IReadOnlyList<StatementSyntax> Statements, RoutineSyntax? Routine, bool Outer)> StatementBlocks()
    {
        foreach (var block in Descend(unit.Statements, null, true)) yield return block;
        foreach (var routine in unit.Routines.Concat(unit.Records.SelectMany(record => record.Methods)))
            foreach (var block in Descend(routine.Statements, routine, true))
                yield return block;

        IEnumerable<(IReadOnlyList<StatementSyntax>, RoutineSyntax?, bool)> Descend(
            IReadOnlyList<StatementSyntax> statements, RoutineSyntax? routine, bool outer)
        {
            token.ThrowIfCancellationRequested();
            yield return (statements, routine, outer);
            foreach (var statement in statements)
            {
                if (statement is NestedRoutineSyntax nested)
                {
                    foreach (var block in Descend(nested.Routine.Statements, nested.Routine, true)) yield return block;
                    continue;
                }
                foreach (var child in Children(statement))
                    foreach (var block in Descend(child, routine, false))
                        yield return block;
            }
        }
    }

    private static ExpressionSyntax? FirstExpression(StatementSyntax statement) => statement switch
    {
        BindingSyntax binding => binding.Initializer,
        ReturnSyntax returned => returned.Expression,
        RaiseSyntax raised => raised.Exception,
        IfSyntax conditional => conditional.Arms[0].Condition,
        CaseStatementSyntax match => match.Selector,
        _ => null
    };

    private static bool EvaluatedFirst(ExpressionSyntax expression, int start, int end,
        List<(int Offset, bool RequiresStatic)> calls, bool allowQualified)
    {
        while (expression.SpanStart <= start && expression.SpanEnd >= end)
        {
            if (expression.SpanStart == start && expression.SpanEnd == end) return true;
            switch (expression)
            {
                case ParenthesizedSyntax parent: expression = parent.Expression; break;
                case UnarySyntax unary: expression = unary.Operand; break;
                case BinarySyntax binary: expression = binary.Left; break;
                case CastSyntax cast: expression = cast.Operand; break;
                case CallSyntax { Arguments.Count: > 0 } call when CallName(call.Target, allowQualified) is { } target:
                    calls.Add(target);
                    expression = call.Arguments[0] is NamedArgumentSyntax named ? named.Value : call.Arguments[0];
                    break;
                default: return false;
            }
        }
        return false;
    }

    /// <summary>Tests whether texts parse exactly as one parameter type and one positional argument.</summary>
    /// <param name="type">The parameter type text.</param>
    /// <param name="argument">The argument expression text.</param>
    /// <param name="token">Cancels the check with an <see cref="OperationCanceledException"/>.</param>
    /// <returns>Whether both fragments are complete and self-contained.</returns>
    public static bool ParameterFragments(string type, string argument, CancellationToken token)
    {
        const string head = "program Fragment; function Value(Item: ";
        const string middle = "): int; begin return 0; end function; begin discard Value(";
        var text = head + type + middle + argument + "); end; end program;";
        try
        {
            var parsed = new RefactoringSyntax(new("<refactoring>", text, token), token).unit;
            if (parsed.Routines is not [{ Parameters: [var parameter] }] ||
                parsed.Statements is not [ExpressionStatementSyntax { Expression: CallSyntax { Arguments: [var value] } }]) return false;
            return parameter.Type.Position == head.Length && parameter.Type.SpanEnd == head.Length + type.Length &&
                value is not (ArgumentModeSyntax or NamedArgumentSyntax) &&
                value.SpanStart == head.Length + type.Length + middle.Length && value.SpanEnd == text.Length - "); end; end program;".Length;
        }
        catch (CompilationFailure) { return false; }
    }

    private static IEnumerable<IReadOnlyList<StatementSyntax>> Children(StatementSyntax statement) => statement switch
    {
        NestedRoutineSyntax nested => [nested.Routine.Statements],
        BlockSyntax block => [block.Statements],
        IfSyntax conditional => conditional.Arms.Select(arm => arm.Statements).Append(conditional.Else),
        WhileSyntax loop => [loop.Body],
        RepeatSyntax loop => [loop.Body],
        ForSyntax loop => [loop.Body],
        ForEachSyntax loop => [loop.Body],
        TrySyntax attempt => attempt.Catches.Select(arm => arm.Body).Prepend(attempt.Body).Append(attempt.Finally ?? []),
        CaseStatementSyntax match => match.Arms.Select(arm => arm.Statements ?? []),
        _ => []
    };

    private int Close(int open)
    {
        if (open < 0) return -1;
        var depth = 0;
        for (var index = open; index < tokens.Count; index++)
        {
            if (Symbol(tokens[index], "(")) depth++;
            if (Symbol(tokens[index], ")") && --depth == 0) return index;
        }
        return -1;
    }

    private int FindIndex(Func<Token, bool> predicate)
    {
        for (var index = 0; index < tokens.Count; index++)
            if (predicate(tokens[index])) return index;
        return -1;
    }

    private static (int Offset, bool RequiresStatic)? CallName(ExpressionSyntax expression, bool allowQualified) => expression switch
    {
        NameExpressionSyntax name => (name.Name.Position, false),
        GenericNameSyntax generic => CallName(generic.Name, allowQualified),
        MemberSyntax member when allowQualified => (member.Name.Position, true),
        _ => null
    };

    private static int End(Token item) => item.Position + item.Length;
    private static bool Symbol(Token item, string value) => item.Kind == TokenKind.Symbol && item.Text == value;
}
