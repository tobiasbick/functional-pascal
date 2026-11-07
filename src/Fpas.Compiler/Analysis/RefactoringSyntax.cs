using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

public sealed record RefactoringRoutine(int Insertion, int FirstOptional, IReadOnlyList<string> Parameters, bool HasVariadicParameter);
public sealed record RefactoringCall(int Close, IReadOnlyList<int> PositionalArguments, bool HasArguments);
public sealed record RefactoringImport(int Start, int Target, int Alias, int End, string TargetName, string AliasName);
public sealed record RefactoringImports(int Start, int End, IReadOnlyList<RefactoringImport> Items, IReadOnlyList<SourceSpan> Comments);
public sealed record RefactoringExtraction(SourceSpan Expression, int Statement, IReadOnlyList<int> PriorCallTargets);

/// <summary>Source extents for edits that must follow FPAS grammar and evaluation order.</summary>
public sealed class RefactoringSyntax
{
    private readonly SourceDocument document;
    private readonly SourceText source;
    private readonly IReadOnlyList<Token> tokens;
    private readonly List<SourceComment> comments = [];
    private readonly CompilationUnitSyntax unit;
    private readonly CancellationToken token;

    public RefactoringSyntax(SourceDocument document, CancellationToken token)
    {
        this.document = document; this.token = token;
        source = new SourceText(document.Path, document.Text, token);
        tokens = new Lexer(source, comments).ReadTokens();
        unit = new Parser(source, tokens).ParseCompilationUnit();
    }

    public RefactoringRoutine? PrivateRoutine(int offset)
    {
        var routine = unit.Routines.Concat(unit.Records.SelectMany(record => record.Methods))
            .SingleOrDefault(item => item.Name.Position == offset && !item.IsPublic &&
            item.Attributes is not { Count: > 0 });
        if (routine is null) return null;
        var open = FindIndex(item => item.Position >= End(routine.TypeParameters?.LastOrDefault() ?? routine.Name) && Symbol(item, "("));
        var close = Close(open);
        if (close < 0) return null;
        var optional = routine.Parameters.TakeWhile(parameter => parameter.Default is null).Count();
        var insertion = optional < routine.Parameters.Count
            ? unit.Declarations[routine.Parameters[optional].Name.Position].Start
            : tokens[close].Position;
        return new(insertion, optional, routine.Parameters.Select(parameter => parameter.Name.Text).ToArray(),
            routine.Parameters.LastOrDefault()?.Mode == "params");
    }

    public RefactoringCall? Call(SourceSpan span, int target)
    {
        var first = FindIndex(item => item.Position == span.Offset);
        var close = FindIndex(item => End(item) == span.Offset + span.Length && Symbol(item, ")"));
        if (first < 0 || close < first || target >= tokens[close].Position) return null;
        var fragment = tokens.Skip(first).Take(close - first + 1).Append(tokens[^1]).ToArray();
        var parsed = new Parser(source, fragment).ParseExpressionFragment();
        return parsed is CallSyntax call && call.SpanEnd == span.Offset + span.Length
            ? new(tokens[close].Position, call.Arguments.TakeWhile(argument => argument is not NamedArgumentSyntax)
                .Select(argument => argument.SpanStart).ToArray(), call.Arguments.Count != 0)
            : null;
    }

    public RefactoringImports? Imports()
    {
        if (unit.Imports.Count == 0) return null;
        var first = FindIndex(item => item.Position == unit.Imports[0].Target.Position);
        var last = FindIndex(item => item.Position == unit.Imports[^1].Alias.Position);
        return new(tokens[first - 1].Position, End(tokens[last + 1]),
            unit.Imports.Select(item => new RefactoringImport(item.Target.Position, item.Target.Parts[^1].Position,
                item.Alias.Position, End(item.Alias), item.Target.Text, item.Alias.Text)).ToArray(),
            comments.Select(comment => document.Span(comment.Start, comment.End - comment.Start)).ToArray());
    }

    public RefactoringExtraction? Extract(int start, int end)
    {
        foreach (var statements in unit.Routines.Concat(unit.Records.SelectMany(record => record.Methods)).Select(routine => routine.Statements)
            .Prepend(unit.Statements))
            if (Search(statements) is { } match) return match;
        return null;

        RefactoringExtraction? Search(IReadOnlyList<StatementSyntax> statements)
        {
            foreach (var statement in statements)
            {
                token.ThrowIfCancellationRequested();
                var (expression, anchor) = statement switch
                {
                    BindingSyntax binding => (binding.Initializer, unit.Declarations[binding.Name.Position].Start),
                    ReturnSyntax returned => (returned.Expression, returned.Position),
                    RaiseSyntax raised => (raised.Exception, raised.Position),
                    IfSyntax conditional => (conditional.Arms[0].Condition, conditional.Position),
                    CaseStatementSyntax match => (match.Selector, match.Position),
                    _ => (null, -1)
                };
                if (expression is not null && First(expression, anchor, []) is { } selection) return selection;
                foreach (var block in Children(statement))
                    if (Search(block) is { } nested) return nested;
            }
            return null;
        }

        RefactoringExtraction? First(ExpressionSyntax expression, int anchor, List<int> priorCalls)
        {
            if (expression.SpanStart <= start && expression.SpanEnd >= end)
            {
                if (expression.SpanStart == start && expression.SpanEnd == end)
                {
                    if (tokens.Any(item => item.Position >= start && item.Position < end &&
                        item.Kind is not (TokenKind.String or TokenKind.Character) &&
                        item.Text is "await" or "ref" or "out" or "in" or "function" or "procedure"))
                        return null;
                    return new(document.Span(start, end - start), anchor, priorCalls);
                }
                switch (expression)
                {
                    case ParenthesizedSyntax parent: return First(parent.Expression, anchor, priorCalls);
                    case UnarySyntax unary: return First(unary.Operand, anchor, priorCalls);
                    case BinarySyntax binary: return First(binary.Left, anchor, priorCalls);
                    case CastSyntax cast: return First(cast.Operand, anchor, priorCalls);
                    case CallSyntax { Arguments.Count: > 0 } call when CallName(call.Target) is { } name:
                        priorCalls.Add(name.Position);
                        var argument = call.Arguments[0] is NamedArgumentSyntax named ? named.Value : call.Arguments[0];
                        return First(argument, anchor, priorCalls);
                }
            }
            return null;
        }
    }

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

    private static Token? CallName(ExpressionSyntax expression) => expression switch
    {
        NameExpressionSyntax name => name.Name,
        GenericNameSyntax generic => CallName(generic.Name),
        _ => null
    };

    private static int End(Token item) => item.Position + item.Length;
    private static bool Symbol(Token item, string value) => item.Kind == TokenKind.Symbol && item.Text == value;
}
