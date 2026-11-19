// [Documentation](../../../docs/pascal/tools/refactoring.md#extract-a-routine)
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Analysis;

/// <summary>The last ordinary binding in a statement selection, when it can initialize a caller local.</summary>
/// <param name="Name">The unescaped local name and its declaration token.</param>
/// <param name="PrefixStart">The start of the binding or its attached comment.</param>
/// <param name="Type">The written annotation of the local.</param>
/// <param name="Initializer">The expression that would become the helper's result.</param>
public sealed record RefactoringRoutineOutput(RefactoringListItem Name, int PrefixStart, SourceSpan Type, SourceSpan Initializer);

/// <summary>The syntax and control flow of one contiguous statement selection.</summary>
/// <param name="Span">Selected statements and their included trivia.</param>
/// <param name="BeforeBegin">Insertion before a routine's outer begin, or null for a helper in the selected block.</param>
/// <param name="TailReturn">Whether the extracted helper returns directly from the enclosing routine.</param>
/// <param name="ReturnType">The enclosing routine's written result annotation, when a tail return supplies a value.</param>
/// <param name="LastBinding">The last ordinary binding, if present.</param>
/// <param name="LiteralContinuations">Starts of lines within multiline string literals, where indentation is significant.</param>
public sealed record RefactoringRoutineRegion(SourceSpan Span, int? BeforeBegin, bool TailReturn, SourceSpan? ReturnType,
    RefactoringRoutineOutput? LastBinding, IReadOnlySet<int> LiteralContinuations);

public sealed partial class RefactoringSyntax
{
    /// <summary>Finds complete, consecutive statements whose control flow can move into a nested routine.</summary>
    /// <param name="start">Inclusive start offset, after trimming whitespace.</param>
    /// <param name="end">Exclusive end offset, before trailing whitespace.</param>
    /// <returns>The editable region, or null for an incomplete or cross-block selection.</returns>
    /// <exception cref="ArgumentException">A complete range crosses a control-flow or resource boundary.</exception>
    public RefactoringRoutineRegion? RoutineRegion(int start, int end)
    {
        if (end <= start ||
            comments.Any(comment => comment.Start < start && start < comment.End || comment.Start < end && end < comment.End))
            return null;
        var selected = tokens.Where(item => item.Position < end && End(item) > start).ToArray();
        if (selected.Length == 0 || selected[0].Position < start || End(selected[^1]) > end)
            return null;
        var first = selected[0].Position;
        var last = End(selected[^1]);
        foreach (var (statements, routine, outer) in StatementBlocks())
        {
            var range = statements.SkipWhile(statement => statement.SpanStart < first).TakeWhile(statement => statement.SpanEnd <= last).ToArray();
            if (range.Length == 0 || range[0].SpanStart != first || range[^1].SpanEnd != last) continue;

            if (selected.Any(item => item.Kind == TokenKind.Keyword && item.Text == "await"))
                throw new ArgumentException("The selection suspends with await.");
            var tail = range[^1] is ReturnSyntax;
            if (tail && (routine?.IsAsync == true || routine is null && unit.IsAsync))
                throw new ArgumentException("An async return cannot move to the synchronous helper.");
            Check(range, root: true, inLoop: false, inCatch: false);

            foreach (var statement in range)
                if (statement is BindingSyntax { Documentation.Lines: [var line, ..] } && line.Start < start)
                    throw new ArgumentException("Include the comment attached to the selected local declaration.");
            var lineEnd = document.Text.IndexOfAny(['\r', '\n'], end);
            var tailText = document.Text.AsSpan(end, (lineEnd < 0 ? document.Text.Length : lineEnd) - end).TrimStart();
            if (tailText.StartsWith("//", StringComparison.Ordinal))
                throw new ArgumentException("Include the selected statement's trailing comment.");

            var beforeBegin = outer && routine is not null ? unit.Declarations[routine.Name.Position].BodyStart : null;
            if (beforeBegin is int begin && tokens.First(item => item.Position == begin).Documentation?.Lines.FirstOrDefault() is { } comment)
                beforeBegin = comment.Start;
            var output = range[^1] is BindingSyntax binding
                ? new RefactoringRoutineOutput(new(binding.Name.Text, Range(binding.Name.Position, End(binding.Name))),
                    binding.Documentation?.Lines.FirstOrDefault()?.Start ?? binding.SpanStart,
                    Range(binding.Type.Position, binding.Type.SpanEnd), Range(binding.Initializer.SpanStart, binding.Initializer.SpanEnd))
                : null;
            var result = tail && routine?.ResultType is { } resultType ? Range(resultType.Position, resultType.SpanEnd) : null;
            var continuations = new HashSet<int>();
            foreach (var literal in selected.Where(item => item.Kind == TokenKind.String))
                for (var offset = literal.Position; offset < End(literal); offset++)
                    if (document.Text[offset] is '\r' or '\n')
                    {
                        if (document.Text[offset] == '\r' && document.Text[offset + 1] == '\n') offset++;
                        if (offset + 1 < End(literal)) continuations.Add(offset + 1);
                    }
            return new(Range(start, end), beforeBegin, tail, result, output, continuations);

            void Check(IEnumerable<StatementSyntax> body, bool root, bool inLoop, bool inCatch)
            {
                foreach (var statement in body)
                {
                    token.ThrowIfCancellationRequested();
                    switch (statement)
                    {
                        case NestedRoutineSyntax:
                            throw new ArgumentException("A selected routine declaration must stay in its declaring scope.");
                        case UsingSyntax when root:
                            throw new ArgumentException("A using declaration must keep the complete block that owns its resource.");
                        case ReturnSyntax when !tail:
                            throw new ArgumentException("A return can move only when the selection ends in a return.");
                        case LoopExitSyntax when !inLoop:
                            throw new ArgumentException("Include the loop targeted by break or continue.");
                        case RaiseSyntax { Exception: null } when !inCatch:
                            throw new ArgumentException("Include the handler targeted by a bare raise.");
                        case TrySyntax attempt:
                            Check(attempt.Body, false, inLoop, inCatch);
                            foreach (var handler in attempt.Catches) Check(handler.Body, false, inLoop, inCatch: true);
                            if (attempt.Finally is { } cleanup) Check(cleanup, false, inLoop, inCatch);
                            break;
                        default:
                            var loop = inLoop || statement is WhileSyntax or RepeatSyntax or ForSyntax or ForEachSyntax;
                            foreach (var child in Children(statement)) Check(child, false, loop, inCatch);
                            break;
                    }
                }
            }
        }
        return null;

        SourceSpan Range(int left, int right) => document.Span(left, right - left);
    }
}
