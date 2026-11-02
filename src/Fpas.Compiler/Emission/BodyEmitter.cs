using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class BodyEmitter(Func<ImportedMethod, EntityHandle> reference, Func<ImportedField, MemberReferenceHandle> fieldReference,
    Func<TypeSymbol, EntityHandle> typeHandle, ReferenceCatalog catalog, Func<string, UserStringHandle> userString, bool debug = false)
{
    private readonly ControlFlowBuilder controlFlow = new();
    private InstructionEncoder il;
    private readonly Stack<(LabelHandle Break, LabelHandle Continue, int Depth)> loops = new();
    private readonly List<TypeSymbol> localTypes = [];
    private int protectedDepth;
    private LabelHandle returned;
    private int resultSlot;
    private int maxStack = 2;
    private int argumentOffset;
    internal List<(int Offset, SourceLocation? Location)> SequencePoints { get; } = [];
    internal List<(int Start, int Length, IReadOnlyList<LocalSymbol> Locals)> LocalScopes { get; } = [];

    internal (InstructionEncoder Code, int MaxStack, IReadOnlyList<TypeSymbol> Locals) Build(BoundRoutine routine)
    {
        il = new(new BlobBuilder(), controlFlow);
        argumentOffset = routine.Symbol.IsInstance ? 1 : 0;
        localTypes.AddRange(routine.Locals.Select(local => local.Captured && local.Mutable ? catalog.CellType(local.Type) : local.Type));
        returned = il.DefineLabel();
        resultSlot = routine.Symbol.Result == TypeSymbol.Void ? -1 : routine.Result?.Slot ?? routine.Locals.Count;
        if (resultSlot >= 0 && routine.Result is null) localTypes.Add(routine.Symbol.Result);
        EmitContracts(routine.Requires);
        EmitStatement(routine.Body);
        il.MarkLabel(returned);
        DebugPoint(null);
        EmitContracts(routine.Ensures);
        if (resultSlot >= 0)
            il.LoadLocal(resultSlot);
        il.OpCode(ILOpCode.Ret);
        return (il, maxStack, localTypes);
    }

    private int Temporary(TypeSymbol type) { localTypes.Add(type); return localTypes.Count - 1; }

    private void EmitContracts(IReadOnlyList<BoundContract>? contracts)
    {
        foreach (var contract in contracts ?? [])
        {
            DebugPoint(contract.Location);
            var passed = il.DefineLabel();
            Evaluate(contract.Condition);
            il.Branch(ILOpCode.Brtrue, passed);
            Evaluate(new BoundFailure(TypeSymbol.Void, contract.Message));
            il.MarkLabel(passed);
            DebugPoint(null);
        }
    }
    private void Jump(LabelHandle target, int depth) => il.Branch(protectedDepth > depth ? ILOpCode.Leave : ILOpCode.Br, target);

    private void Evaluate(BoundExpression expression)
    {
        maxStack = Math.Max(maxStack, ExpressionStack.Depth(expression) + 2);
        EmitExpression(expression);
    }

    private void EmitStatement(BoundStatement statement)
    {
        switch (statement)
        {
            case BoundSequencePoint point:
                DebugPoint(point.Location);
                EmitStatement(point.Statement);
                DebugPoint(null);
                break;
            case BoundBlock block:
                var scopeStart = il.Offset;
                foreach (var child in block.Statements)
                    EmitStatement(child);
                if (debug && block.ScopeLocals is { Count: > 0 } && il.Offset > scopeStart)
                    LocalScopes.Add((scopeStart, il.Offset - scopeStart, block.ScopeLocals));
                break;
            case BoundBinding binding:
                StoreStorage(binding.Local, () =>
                {
                    Evaluate(binding.Initializer);
                    if (binding.Local.Captured && binding.Local.Mutable) { il.OpCode(ILOpCode.Newobj); il.Token(reference(catalog.CellConstructor(binding.Local.Type))); }
                });
                break;
            case BoundDispose dispose: EmitDispose(dispose.Resource); break;
            case BoundAssignment assignment:
                EmitStore(assignment.Target, assignment.Value);
                break;
            case BoundExpressionStatement expression:
                Evaluate(expression.Expression);
                if (expression.Discard)
                    il.OpCode(ILOpCode.Pop);
                break;
            case BoundReturn exit:
                if (exit.Expression is not null)
                {
                    Evaluate(exit.Expression);
                    il.StoreLocal(resultSlot);
                }
                Jump(returned, 0);
                break;
            case BoundIf conditional:
                var done = il.DefineLabel();
                foreach (var arm in conditional.Arms)
                {
                    var next = il.DefineLabel();
                    Evaluate(arm.Condition);
                    il.Branch(ILOpCode.Brfalse, next);
                    EmitStatement(arm.Body);
                    il.Branch(ILOpCode.Br, done);
                    il.MarkLabel(next);
                }
                EmitStatement(conditional.Else);
                il.MarkLabel(done);
                break;
            case BoundWhile loop:
                var condition = il.DefineLabel();
                var end = il.DefineLabel();
                loops.Push((end, condition, protectedDepth));
                il.MarkLabel(condition);
                Evaluate(loop.Condition);
                il.Branch(ILOpCode.Brfalse, end);
                EmitStatement(loop.Body);
                il.Branch(ILOpCode.Br, condition);
                il.MarkLabel(end);
                loops.Pop();
                break;
            case BoundFor loop:
                EmitFor(loop);
                break;
            case BoundLoopExit exit:
                var loopExit = loops.Peek();
                Jump(exit.IsBreak ? loopExit.Break : loopExit.Continue, loopExit.Depth);
                break;
            case BoundUsing resource:
                Evaluate(resource.Initializer);
                il.StoreLocal(resource.Local.Slot);
                EmitFinally(() => EmitStatement(resource.Body), () => EmitDispose(resource));
                break;
            case BoundTry guarded:
                if (guarded.Finally is null) EmitCatches(guarded);
                else EmitFinally(() => EmitCatches(guarded), () => EmitStatement(guarded.Finally));
                break;
            case BoundThrow raised:
                if (raised.Exception is null) il.OpCode(ILOpCode.Rethrow);
                else { Evaluate(raised.Exception); il.OpCode(ILOpCode.Throw); }
                break;
            default:
                throw new InvalidOperationException("Unrecognized bound statement.");
        }
    }

    private void DebugPoint(SourceLocation? location)
    {
        if (!debug) return;
        if (SequencePoints.Count > 0 && SequencePoints[^1].Offset == il.Offset) SequencePoints[^1] = (il.Offset, location);
        else SequencePoints.Add((il.Offset, location));
        if (location is not null) il.OpCode(ILOpCode.Nop);
    }

    private void EmitFor(BoundFor loop)
    {
        var body = il.DefineLabel();
        var step = il.DefineLabel();
        var end = il.DefineLabel();
        Evaluate(loop.Lower);
        il.StoreLocal(loop.Counter.Slot);
        Evaluate(loop.Upper);
        il.StoreLocal(loop.Limit.Slot);
        il.LoadLocal(loop.Counter.Slot);
        il.LoadLocal(loop.Limit.Slot);
        il.Branch(loop.Descending ? ILOpCode.Blt : ILOpCode.Bgt, end);
        loops.Push((end, step, protectedDepth));
        il.MarkLabel(body);
        EmitStatement(loop.Body);
        il.MarkLabel(step);
        il.LoadLocal(loop.Counter.Slot);
        il.LoadLocal(loop.Limit.Slot);
        il.Branch(ILOpCode.Beq, end);
        il.LoadLocal(loop.Counter.Slot);
        il.LoadConstantI4(1);
        il.OpCode(loop.Descending ? ILOpCode.Sub_ovf : ILOpCode.Add_ovf);
        il.StoreLocal(loop.Counter.Slot);
        il.Branch(ILOpCode.Br, body);
        il.MarkLabel(end);
        loops.Pop();
    }
}
