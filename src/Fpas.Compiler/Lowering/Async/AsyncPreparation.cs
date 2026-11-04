using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;

namespace Fpas.Compiler.Lowering.Async;

internal static class AsyncPreparation
{
    internal static Dictionary<BoundRoutine, AsyncPlan> Prepare(BoundCompilation program, ReferenceCatalog references, CancellationToken cancellationToken)
    {
        var plans = new Dictionary<BoundRoutine, AsyncPlan>(ReferenceEqualityComparer.Instance);
        foreach (var unit in program.Units)
        {
            var routines = unit.Routines.Concat(unit.Records.SelectMany(record => record.Bodies.Values))
                .Concat(unit.Closures.SelectMany(closure => closure.Bodies));
            foreach (var routine in routines)
            {
                cancellationToken.ThrowIfCancellationRequested();
                if (routine.Symbol.IsAsync) plans.Add(routine, AsyncLowering.Lower(routine, references));
            }
        }
        return plans;
    }
}
