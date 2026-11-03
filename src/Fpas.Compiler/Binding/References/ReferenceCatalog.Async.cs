using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal BoundAwait Await(BoundExpression operand, SourceText source, int position)
    {
        var name = operand.Type.Definition?.FullName;
        if (name is not ("System.Threading.Tasks.Task" or "System.Threading.Tasks.Task`1" or "System.Threading.Tasks.ValueTask" or "System.Threading.Tasks.ValueTask`1" or
            "System.Runtime.CompilerServices.ConfiguredTaskAwaitable" or "System.Runtime.CompilerServices.ConfiguredTaskAwaitable`1" or
            "System.Runtime.CompilerServices.ConfiguredValueTaskAwaitable" or "System.Runtime.CompilerServices.ConfiguredValueTaskAwaitable`1"))
            throw source.Error("FP3030", "await supports Task, ValueTask, and their standard ConfigureAwait awaitables.", position);
        var getAwaiter = FindMethod(operand.Type, "GetAwaiter", [], position, source, false);
        var completed = Property(getAwaiter.Result, "IsCompleted", false)?.Getter ?? throw source.Error("FP3030", "Missing awaiter completion metadata.", position);
        return new(operand, getAwaiter, completed, FindMethod(getAwaiter.Result, "GetResult", [], position, source, false), source.StatementLocation(position));
    }
}
