using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Lowering;

internal sealed class AsyncPlan(BoundRoutine routine)
{
    internal List<LocalSymbol> Locals { get; } = routine.Locals.ToList();
    internal List<AsyncBlock> Blocks { get; } = [];
    internal int Entry { get; set; }
    internal LocalSymbol? Result { get; set; } = routine.Result;
}

internal sealed class AsyncBlock(int id, int onException, AsyncExit exit, IReadOnlyList<BoundStatement>? statements = null)
{
    internal int Id { get; } = id;
    internal int OnException { get; } = onException;
    internal AsyncExit Exit { get; set; } = exit;
    internal IReadOnlyList<BoundStatement> Statements { get; } = statements ?? [];
    internal SourceLocation? Location { get; set; }
}

internal abstract record AsyncExit;
internal sealed record AsyncGoto(int Target) : AsyncExit;
internal sealed record AsyncBranch(BoundExpression Condition, int True, int False) : AsyncExit;
internal sealed record AsyncSuspend(LocalSymbol Awaiter, ImportedMethod IsCompleted, int Resume, SourceLocation Location) : AsyncExit;
internal sealed record AsyncPropagate(BoundExpression Exception, int Target) : AsyncExit;
internal sealed record AsyncRaise(BoundExpression Exception) : AsyncExit;
internal sealed record AsyncComplete() : AsyncExit;
internal sealed record AsyncFail() : AsyncExit;
