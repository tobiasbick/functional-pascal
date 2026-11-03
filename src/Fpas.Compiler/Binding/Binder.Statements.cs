using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private BoundBlock BindBlock(IReadOnlyList<StatementSyntax> statements, bool newScope = true)
    {
        if (newScope)
            scopes.Push(new(StringComparer.Ordinal));
        try
        {
            var nested = PrepareNestedRoutines(statements);
            var bound = new List<BoundStatement>();
            var flow = ExitFlow.Next;
            for (var index = 0; index < statements.Count; index++)
            {
                var statement = statements[index];
                if (statement is NestedRoutineSyntax) continue;
                if (!flow.HasFlag(ExitFlow.Next))
                    throw source.Error("FP3008", "This statement is unreachable.", statement.Position);
                var item = statement is UsingSyntax resource
                    ? BindUsing(resource, statements.Skip(index + 1).ToArray()) : BindStatement(statement);
                bound.Add(new BoundSequencePoint(source.StatementLocation(statement.Position), item));
                flow = flow & ~ExitFlow.Next | FlowAnalysis.Of(item);
                if (statement is UsingSyntax) break;
            }
            if (nested is not null) BindNestedBodies(nested);
            return new(bound, scopes.Peek().Values.ToArray());
        }
        finally
        {
            if (newScope)
                foreach (var local in scopes.Pop().Values) nonNull.Remove(local);
        }
    }

    private BoundStatement BindStatement(StatementSyntax statement)
    {
        switch (statement)
        {
            case BindingSyntax binding:
                var type = BindType(binding.Type);
                var original = BindExpression(binding.Initializer, expected: type);
                var initializer = Convert(original, type, binding.Initializer.Position);
                var local = Allocate(binding.Name.Text, type, binding.Mutable);
                local.Captured = local.Mutable && potentialNestedCaptures.Contains(local.Name);
                local.Documentation = binding.Documentation;
                Declare(local, binding.Name.Position);
                UpdateFact(local, original);
                return new BoundBinding(local, initializer);
            case AssignmentSyntax assignment:
                BoundExpression target;
                if (assignment.Target is NameExpressionSyntax name)
                {
                    if (!HasValue(name.Name.Text)) throw source.Error("FP3009", "Assignment requires writable storage.", name.Position);
                    target = Value(name.Name.Text, name.Position);
                    if (!Writable(target)) throw source.Error("FP3009", "Assignment requires writable storage.", name.Position);
                    if (target is BoundLocal stored) target = new BoundLocal(stored.Local);
                    if (target is BoundCapture capture) target = new BoundCapture(capture.Capture);
                    if (target is BoundParameter parameter) target = new BoundParameter(parameter.Parameter);
                }
                else if (assignment.Target is MemberSyntax member) target = BindMember(member, writing: true);
                else if (assignment.Target is IndexSyntax index) target = BindIndex(index, check: true, writing: true);
                else throw source.Error("FP3009", "Assignment requires writable storage.", assignment.Position);
                var assigned = BindExpression(assignment.Value, expected: target.Type);
                var converted = Convert(assigned, target.Type, assignment.Value.Position);
                if (target is BoundLocal assignedLocal) UpdateFact(assignedLocal.Local, assigned);
                if (target is BoundCapture assignedCapture) { nonNull.Remove(assignedCapture.Capture); if (assigned.Type.CanHaveNull && !assigned.Type.NeedsNullCheck && assigned.Type != TypeSymbol.Null) nonNull.Add(assignedCapture.Capture); }
                if (target is BoundParameter assignedParameter) { nonNull.Remove(assignedParameter.Parameter); if (assigned.Type.CanHaveNull && !assigned.Type.NeedsNullCheck && assigned.Type != TypeSymbol.Null) nonNull.Add(assignedParameter.Parameter); }
                return new BoundAssignment(target, converted);
            case ExpressionStatementSyntax expression:
                var value = BindExpression(expression.Expression);
                if (value is BoundMethodGroup) throw source.Error("FP3019", "An overloaded method group needs a delegate target.", expression.Position);
                if (!expression.Discard && value.Type != TypeSymbol.Void || expression.Discard && value.Type == TypeSymbol.Void)
                    throw source.Error("FP3006", "Consume a returned value or use discard; an ordinary expression statement must be a void call.", expression.Position);
                return new BoundExpressionStatement(value, expression.Discard);
            case ReturnSyntax returned:
                if (finallyBoundaries.Count != 0) throw source.Error("FP3017", "return cannot leave a finally body.", returned.Position);
                if (currentRoutine.BodyResult == TypeSymbol.Void)
                {
                    if (returned.Expression is not null)
                        throw source.Error("FP3002", "A procedure or program entry cannot return a value.", returned.Position);
                    return new BoundReturn(null);
                }
                if (returned.Expression is null)
                    throw source.Error("FP3002", "A function return requires a value.", returned.Position);
                return new BoundReturn(Convert(BindExpression(returned.Expression, expected: currentRoutine.BodyResult), currentRoutine.BodyResult, returned.Position));
            case BlockSyntax block:
                return BindBlock(block.Statements);
            case CaseStatementSyntax match: return BindCaseStatement(match);
            case IfSyntax conditional:
                return BindIf(conditional);
            case WhileSyntax loop: return BindWhile(loop);
            case RepeatSyntax loop: return BindRepeat(loop);
            case TrySyntax guarded:
                return BindTry(guarded);
            case RaiseSyntax raised:
                if (raised.Exception is null)
                {
                    if (catchDepth == 0 || finallyBoundaries.TryPeek(out var boundary) && catchDepth <= boundary.CatchDepth)
                        throw source.Error("FP3017", "A rethrow requires an active lexical catch body.", raised.Position);
                    return new BoundThrow(null);
                }
                return new BoundThrow(Convert(BindExpression(raised.Exception), references.Symbol(references.Required("System.Exception")), raised.Position));
            case ForEachSyntax loop: return BindForEach(loop);
            case ForSyntax loop:
                return BindFor(loop);
            case LoopExitSyntax exit:
                if (finallyBoundaries.TryPeek(out var finallyBoundary) && loopDepth <= finallyBoundary.LoopDepth)
                    throw source.Error("FP3017", "Loop exits cannot leave a finally body.", exit.Position);
                if (loopDepth == 0)
                    throw source.Error("FP3010", "break and continue require an enclosing loop.", exit.Position);
                return new BoundLoopExit(exit.IsBreak);
            default:
                throw new InvalidOperationException("Unrecognized statement syntax.");
        }
    }

}
