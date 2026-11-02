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
            var bound = new List<BoundStatement>();
            var flow = ExitFlow.Next;
            for (var index = 0; index < statements.Count; index++)
            {
                var statement = statements[index];
                if (!flow.HasFlag(ExitFlow.Next))
                    throw source.Error("FP3008", "This statement is unreachable.", statement.Position);
                var item = statement is UsingSyntax resource
                    ? BindUsing(resource, statements.Skip(index + 1).ToArray()) : BindStatement(statement);
                bound.Add(new BoundSequencePoint(source.StatementLocation(statement.Position), item));
                flow = flow & ~ExitFlow.Next | FlowAnalysis.Of(item);
                if (statement is UsingSyntax) break;
            }
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
            case WhileSyntax loop:
                InvalidateAssigned(loop.Body);
                var incoming = nonNull.ToHashSet();
                var condition = Condition(loop.Condition);
                SetFacts(Refine(loop.Condition, true, incoming));
                loopDepth++;
                BoundBlock body;
                try { body = BindBlock(loop.Body); }
                finally { loopDepth--; }
                SetFacts(FlowAnalysis.Of(body).HasFlag(ExitFlow.Break) ? incoming : Refine(loop.Condition, false, incoming));
                return new BoundWhile(condition, body);
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

    private BoundFor BindFor(ForSyntax loop)
    {
        if (BindType(loop.Type) != TypeSymbol.Int32)
            throw source.Error("FP3002", "Counting loops currently require an int counter.", loop.Type.Position);
        var lower = Convert(BindExpression(loop.Lower), TypeSymbol.Int32, loop.Lower.Position);
        var upper = Convert(BindExpression(loop.Upper), TypeSymbol.Int32, loop.Upper.Position);
        InvalidateAssigned(loop.Body);
        var incoming = nonNull.ToHashSet();
        scopes.Push(new(StringComparer.Ordinal));
        loopDepth++;
        try
        {
            var counter = Allocate(loop.Name.Text, TypeSymbol.Int32);
            Declare(counter, loop.Name.Position);
            var limit = Allocate("<ForLimit>", TypeSymbol.Int32);
            return new(counter, limit, lower, upper, loop.Descending, BindBlock(loop.Body, newScope: false));
        }
        finally
        {
            loopDepth--;
            foreach (var local in scopes.Pop().Values) nonNull.Remove(local);
            SetFacts(incoming);
        }
    }
}
