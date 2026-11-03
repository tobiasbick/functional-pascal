using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed class ParameterFlow(SourceText source, int position)
{
    private sealed class Paths : Dictionary<ExitFlow, HashSet<ParameterSymbol>>
    {
        internal void AddPath(ExitFlow exit, IEnumerable<ParameterSymbol> assigned)
        {
            if (TryGetValue(exit, out var current)) current.IntersectWith(assigned);
            else Add(exit, assigned.ToHashSet());
        }
        internal void Include(Paths paths)
        {
            foreach (var (exit, assigned) in paths) AddPath(exit, assigned);
        }
    }

    internal void Validate(BoundRoutine routine)
    {
        var assigned = routine.Symbol.Parameters.Where(parameter => parameter.Mode != "out").ToHashSet();
        foreach (var clause in routine.Requires ?? []) Expression(clause.Condition, assigned);
        var paths = Statement(routine.Body, assigned);
        foreach (var (exit, state) in paths)
        {
            if (exit is not (ExitFlow.Next or ExitFlow.Return)) continue;
            foreach (var parameter in routine.Symbol.Parameters.Where(parameter => parameter.Mode == "out"))
                if (!state.Contains(parameter)) throw source.Error("FP3028", $"Out parameter '{parameter.Name}' must be assigned on every successful exit.", position);
            foreach (var clause in routine.Ensures ?? []) Expression(clause.Condition, state);
        }
    }

    private Paths Statement(BoundStatement statement, HashSet<ParameterSymbol> input)
    {
        var state = input.ToHashSet();
        var result = new Paths();
        switch (statement)
        {
            case BoundSequencePoint point: return Statement(point.Statement, state);
            case BoundBlock block:
                result.AddPath(ExitFlow.Next, state);
                foreach (var child in block.Statements)
                {
                    if (!result.Remove(ExitFlow.Next, out var next)) break;
                    result.Include(Statement(child, next));
                }
                return result;
            case BoundBinding binding: Expression(binding.Initializer, state); break;
            case BoundAssignment assignment:
                Address(assignment.Target, state);
                Expression(assignment.Value, state);
                if (assignment.Target is BoundParameter parameter) state.Add(parameter.Parameter);
                break;
            case BoundExpressionStatement expression: Expression(expression.Expression, state); break;
            case BoundReturn returned:
                if (returned.Expression is not null) Expression(returned.Expression, state);
                result.AddPath(ExitFlow.Return, state); return result;
            case BoundThrow raised:
                if (raised.Exception is not null) Expression(raised.Exception, state);
                result.AddPath(ExitFlow.Throw, state); return result;
            case BoundLoopExit exit:
                result.AddPath(exit.IsBreak ? ExitFlow.Break : ExitFlow.Continue, state); return result;
            case BoundIf conditional:
                foreach (var arm in conditional.Arms)
                {
                    if (arm.Condition is BoundBoolean { Value: false }) continue;
                    Expression(arm.Condition, state);
                    result.Include(Statement(arm.Body, state));
                    if (arm.Condition is BoundBoolean { Value: true }) return result;
                }
                result.Include(Statement(conditional.Else, state)); return result;
            case BoundWhile loop:
                Expression(loop.Condition, state);
                return Loop(loop.Body, state, loop.Condition is BoundBoolean { Value: true });
            case BoundFor loop:
                Expression(loop.Lower, state); Expression(loop.Upper, state);
                return Loop(loop.Body, state);
            case BoundRepeat loop: return Repeat(loop, state);
            case BoundUsing resource:
                Expression(resource.Initializer, state);
                return Statement(resource.Body, state);
            case BoundTry guarded:
                result.Include(Statement(guarded.Body, state));
                foreach (var caught in guarded.Catches)
                {
                    var entry = state.ToHashSet();
                    if (caught.Filter is not null) Expression(caught.Filter, entry);
                    result.Include(Statement(caught.Body, entry));
                }
                if (guarded.Finally is null) return result;
                var cleaned = new Paths();
                foreach (var (exit, before) in result)
                {
                    var cleanup = Statement(guarded.Finally, before);
                    if (cleanup.Remove(ExitFlow.Next, out var after)) cleaned.AddPath(exit, after);
                    cleaned.Include(cleanup);
                }
                return cleaned;
            default: throw new InvalidOperationException("Unrecognized statement in parameter flow.");
        }
        result.AddPath(ExitFlow.Next, state);
        return result;
    }

    private Paths Loop(BoundStatement body, HashSet<ParameterSymbol> state, bool forever = false)
    {
        var paths = Statement(body, state);
        paths.Remove(ExitFlow.Next); paths.Remove(ExitFlow.Continue);
        if (paths.Remove(ExitFlow.Break, out var stopped)) paths.AddPath(ExitFlow.Next, stopped);
        if (!forever) paths.AddPath(ExitFlow.Next, state);
        return paths;
    }

    private Paths Repeat(BoundRepeat loop, HashSet<ParameterSymbol> state)
    {
        var paths = Statement(loop.Body, state);
        HashSet<ParameterSymbol>? condition = null;
        foreach (var exit in new[] { ExitFlow.Next, ExitFlow.Continue })
            if (paths.Remove(exit, out var arrived))
                if (condition is null) condition = arrived; else condition.IntersectWith(arrived);
        if (paths.Remove(ExitFlow.Break, out var stopped)) paths.AddPath(ExitFlow.Next, stopped);
        if (condition is not null)
        {
            Expression(loop.Condition, condition);
            if (loop.Condition is not BoundBoolean { Value: false }) paths.AddPath(ExitFlow.Next, condition);
        }
        return paths;
    }

    private void Address(BoundExpression target, HashSet<ParameterSymbol> state)
    {
        switch (target)
        {
            case BoundParameter or BoundLocal or BoundCapture: break;
            case BoundArrayIndex index:
                Expression(index.Receiver, state);
                foreach (var argument in index.Indices) Expression(argument, state);
                break;
            case BoundField field:
                if (field.Receiver is not null) Expression(field.Receiver, state);
                break;
            case BoundProperty property:
                if (property.Receiver is not null) Expression(property.Receiver, state);
                foreach (var argument in property.Arguments ?? []) Expression(argument, state);
                break;
            default: Expression(target, state); break;
        }
    }

    private void Expression(BoundExpression expression, HashSet<ParameterSymbol> state)
    {
        switch (expression)
        {
            case BoundAwait awaited: Expression(awaited.Operand, state); break;
            case BoundParameter { Parameter.Mode: "out" } parameter:
                if (!state.Contains(parameter.Parameter)) throw source.Error("FP3028", $"Out parameter '{parameter.Parameter.Name}' cannot be read before assignment.", position);
                break;
            case BoundAddress address:
                if (address.Mode == "out") Address(address.Target, state); else Expression(address.Target, state);
                break;
            case BoundDereference dereference: Expression(dereference.Address, state); break;
            case BoundCall call:
                if (call.Receiver is not null) Expression(call.Receiver, state);
                foreach (var argument in call.Arguments) Expression(argument, state);
                state.UnionWith(call.WrittenParameters ?? []);
                break;
            case BoundNew creation:
                foreach (var argument in creation.Arguments) Expression(argument, state);
                state.UnionWith(creation.WrittenParameters ?? []);
                break;
            case BoundSequence sequence:
                foreach (var binding in sequence.Bindings) Expression(binding.Initializer, state);
                Expression(sequence.Value, state); break;
            case BoundBinary binary:
                Expression(binary.Left, state);
                var right = state.ToHashSet(); Expression(binary.Right, right);
                if (binary.Left.Type == TypeSymbol.Boolean && binary.Operator is "and" or "or") state.IntersectWith(right);
                else state.UnionWith(right);
                break;
            case BoundCoalesce coalesce:
                Expression(coalesce.Left, state);
                var fallback = state.ToHashSet(); Expression(coalesce.Right, fallback); state.IntersectWith(fallback);
                break;
            case BoundConditional conditional:
                var exits = new List<HashSet<ParameterSymbol>>();
                foreach (var arm in conditional.Arms)
                {
                    Expression(arm.Condition, state);
                    var branch = state.ToHashSet(); Expression(arm.Value, branch); exits.Add(branch);
                }
                Expression(conditional.Else, state);
                foreach (var branch in exits) state.IntersectWith(branch);
                break;
            case BoundUnary unary: Expression(unary.Operand, state); break;
            case BoundConversion conversion: Expression(conversion.Operand, state); break;
            case BoundNullTest test: Expression(test.Operand, state); break;
            case BoundTypeTest test: Expression(test.Value, state); break;
            case BoundSafeCast cast: Expression(cast.Value, state); break;
            case BoundField field: if (field.Receiver is not null) Expression(field.Receiver, state); break;
            case BoundRecordField field: Expression(field.Receiver, state); break;
            case BoundProperty property:
                if (property.Receiver is not null) Expression(property.Receiver, state);
                foreach (var argument in property.Arguments ?? []) Expression(argument, state);
                break;
            case BoundArrayLength length: Expression(length.Receiver, state); break;
            case BoundArrayIndex index:
                Expression(index.Receiver, state); foreach (var argument in index.Indices) Expression(argument, state); break;
            case BoundArrayLiteral array: foreach (var element in array.Elements) Expression(element, state); break;
            case BoundNewArray array: foreach (var length in array.Lengths) Expression(length, state); break;
            case BoundLambda lambda: foreach (var capture in lambda.Closure.Captures) Expression(capture.Value, state); break;
            case BoundEnvironment environment: foreach (var argument in environment.Arguments) Expression(argument, state); break;
            case BoundMethodDelegate method: if (method.Receiver is not null) Expression(method.Receiver, state); break;
        }
    }
}
