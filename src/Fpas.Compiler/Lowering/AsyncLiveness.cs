using Fpas.Compiler.Binding;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Lowering;

internal static class AsyncLiveness
{
    internal static void Validate(AsyncPlan plan, SourceText source)
    {
        var incoming = plan.Blocks.Select(_ => new HashSet<LocalSymbol>()).ToArray();
        var outgoing = plan.Blocks.Select(_ => new HashSet<LocalSymbol>()).ToArray();
        var changed = true;
        while (changed)
        {
            changed = false;
            foreach (var block in plan.Blocks.AsEnumerable().Reverse())
            {
                var live = Successors(block).Where(target => target >= 0).SelectMany(target => incoming[target]).ToHashSet();
                outgoing[block.Id] = live.ToHashSet();
                switch (block.Exit)
                {
                    case AsyncBranch branch: Uses(branch.Condition, live); break;
                    case AsyncSuspend awaited: live.Add(awaited.Awaiter); break;
                    case AsyncPropagate propagated: Uses(propagated.Exception, live); break;
                    case AsyncRaise raised: Uses(raised.Exception, live); break;
                }
                foreach (var statement in block.Statements.Reverse())
                {
                    switch (statement)
                    {
                        case BoundBinding binding: live.Remove(binding.Local); Uses(binding.Initializer, live); break;
                        case BoundAssignment assignment:
                            if (assignment.Target is BoundLocal target) live.Remove(target.Local); else Uses(assignment.Target, live);
                            Uses(assignment.Value, live); break;
                        case BoundExpressionStatement expression: Uses(expression.Expression, live); break;
                        case BoundDispose resource: live.Add(resource.Resource.Local); break;
                    }
                }
                if (!incoming[block.Id].SetEquals(live)) { incoming[block.Id] = live; changed = true; }
            }
        }
        foreach (var block in plan.Blocks)
            if (block.Exit is AsyncSuspend awaited && outgoing[block.Id].Any(local => local.Type.Kind == TypeKind.ByRef))
                throw new CompilationFailure(new("FP3030", "A managed reference cannot remain live across await; evaluate the awaited value before taking the reference.", awaited.Location));
    }

    private static IEnumerable<int> Successors(AsyncBlock block)
    {
        yield return block.OnException;
        switch (block.Exit)
        {
            case AsyncGoto jump: yield return jump.Target; break;
            case AsyncBranch branch: yield return branch.True; yield return branch.False; break;
            case AsyncSuspend awaited: yield return awaited.Resume; break;
            case AsyncPropagate propagated: yield return propagated.Target; break;
        }
    }

    private static void Uses(BoundExpression expression, HashSet<LocalSymbol> locals)
    {
        if (expression is BoundLocal local) locals.Add(local.Local);
        if (expression is BoundLocalStorage storage) locals.Add(storage.Local);
        if (expression is BoundSequence sequence)
        {
            Uses(sequence.Value, locals);
            foreach (var binding in sequence.Bindings.Reverse()) { locals.Remove(binding.Local); Uses(binding.Initializer, locals); }
            return;
        }
        foreach (var child in BoundTree.Children(expression)) Uses(child, locals);
    }
}
