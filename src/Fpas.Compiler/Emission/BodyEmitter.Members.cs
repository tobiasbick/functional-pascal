using System.Reflection.Metadata;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class BodyEmitter
{
    private ImportedMethod NullableMethod(TypeSymbol type, string name, IReadOnlyList<TypeSymbol>? arguments = null)
        => catalog.FindMethod(type, name, arguments ?? [], 0, new Syntax.SourceText("<lowering>", ""), isStatic: false);

    private void EmitDefault(TypeSymbol type)
    {
        if (type.IsReference) { il.OpCode(ILOpCode.Ldnull); return; }
        var slot = Temporary(type);
        il.LoadLocalAddress(slot);
        il.OpCode(ILOpCode.Initobj); il.Token(typeHandle(type));
        il.LoadLocal(slot);
    }

    private void EmitAddress(BoundExpression expression, bool force = false)
    {
        if (expression is BoundDereference dereference) { EmitExpression(dereference.Address); return; }
        if (expression is BoundParameter parameter && (force || parameter.Parameter.Mode is "ref" or "out"))
        {
            if (asyncMachine is not null) { il.LoadArgument(0); il.OpCode(ILOpCode.Ldflda); il.Token(fieldReference(catalog.MachineParameter(asyncMachine, parameter.Parameter))); return; }
            if (parameter.Parameter.ByRef) il.LoadArgument(parameter.Parameter.Index + argumentOffset);
            else il.LoadArgumentAddress(parameter.Parameter.Index + argumentOffset);
            return;
        }
        if (expression is BoundLocal local && (force || local.Local.Mutable))
        {
            if (local.Local.Captured && local.Local.Mutable) { LoadStorage(local.Local); il.OpCode(ILOpCode.Ldflda); il.Token(fieldReference(catalog.CellValue(local.Local.Type))); }
            else if (Hoisted(local.Local)) { il.LoadArgument(0); il.OpCode(ILOpCode.Ldflda); il.Token(fieldReference(catalog.MachineLocal(asyncMachine!, local.Local))); }
            else il.LoadLocalAddress(LocalSlot(local.Local));
            return;
        }
        if (expression is BoundCapture { Capture.Mutable: true } capture)
        {
            EmitCaptureStorage(capture.Capture); il.OpCode(ILOpCode.Ldflda); il.Token(fieldReference(catalog.CellValue(capture.Capture.Type))); return;
        }
        if (expression is BoundArrayIndex index) { EmitArrayAddress(index); return; }
        if (expression is BoundField field && (force || !field.Field.IsReadOnly) &&
            (force || field.Receiver is null || field.Receiver.Type.IsReference || WritableAddress(field.Receiver)))
        {
            if (field.Receiver is not null)
            {
                if (field.Receiver.Type.IsValue) EmitAddress(field.Receiver, force); else EmitReferenceReceiver(field.Receiver);
            }
            il.OpCode(field.Field.IsStatic ? ILOpCode.Ldsflda : ILOpCode.Ldflda); il.Token(fieldReference(field.Field));
            return;
        }
        var temporary = Temporary(expression.Type);
        EmitExpression(expression); il.StoreLocal(temporary); il.LoadLocalAddress(temporary);
    }

    private static bool WritableAddress(BoundExpression expression) => expression is BoundArrayIndex or BoundCapture { Capture.Mutable: true } or BoundDereference or BoundParameter { Parameter.Mode: "ref" or "out" } || expression is BoundLocal { Local.Mutable: true } ||
        expression is BoundField { Field.IsReadOnly: false } field && (field.Receiver is null || field.Receiver.Type.IsReference || WritableAddress(field.Receiver));

    private void EmitCall(ImportedMethod method, IReadOnlyList<BoundExpression> arguments, BoundExpression? receiver)
    {
        if (receiver is not null)
        {
            if (receiver.Type.IsValue || receiver.Type.IsParameter) EmitAddress(receiver); else EmitExpression(receiver);
        }
        foreach (var argument in arguments) EmitExpression(argument);
        if (method.ConstrainedType is { } implementor)
        {
            il.OpCode(ILOpCode.Constrained); il.Token(typeHandle(implementor));
            il.Call(reference(method));
            return;
        }
        var virtualCall = receiver is not null && (receiver.Type.IsReference || receiver.Type.IsParameter && !receiver.Type.NullableValue || method.IsVirtual || method.Owner.IsReference);
        if (virtualCall && (receiver!.Type.IsValue || receiver.Type.IsParameter)) { il.OpCode(ILOpCode.Constrained); il.Token(typeHandle(receiver.Type)); }
        if (virtualCall) { il.OpCode(ILOpCode.Callvirt); il.Token(reference(method)); }
        else il.Call(reference(method));
    }

    private void EmitReferenceReceiver(BoundExpression receiver)
    {
        EmitExpression(receiver);
        if (receiver.Type.IsParameter) { il.OpCode(ILOpCode.Box); il.Token(typeHandle(receiver.Type)); }
    }

    private void EmitField(BoundField field)
    {
        if (field.Field.IsLiteral)
        {
            switch (field.Field.Constant)
            {
                case string text: il.LoadString(userString(text)); break;
                case null: il.OpCode(ILOpCode.Ldnull); break;
                default: EmitNumber(field.Field.Constant); break;
            }
            return;
        }
        if (field.Receiver is not null)
        {
            if (field.Receiver.Type.IsValue) EmitAddress(field.Receiver); else EmitReferenceReceiver(field.Receiver);
        }
        il.OpCode(field.Field.IsStatic ? ILOpCode.Ldsfld : ILOpCode.Ldfld); il.Token(fieldReference(field.Field));
    }

    private void EmitStore(BoundExpression target, BoundExpression value)
    {
        maxStack = Math.Max(maxStack, Math.Max(ExpressionStack.Depth(target), ExpressionStack.Depth(value) + 1) + 2);
        if (target is BoundParameter parameter)
        {
            il.LoadArgument(parameter.Parameter.Index + argumentOffset); EmitExpression(value);
            il.OpCode(ILOpCode.Stobj); il.Token(typeHandle(parameter.Parameter.Type)); return;
        }
        if (target is BoundLocal local)
        {
            if (local.Local.Captured && local.Local.Mutable)
            {
                LoadStorage(local.Local); EmitExpression(value); il.OpCode(ILOpCode.Stfld); il.Token(fieldReference(catalog.CellValue(local.Local.Type)));
            }
            else StoreStorage(local.Local, () => EmitExpression(value));
            return;
        }
        if (target is BoundCapture capture)
        {
            EmitCaptureStorage(capture.Capture); EmitExpression(value); il.OpCode(ILOpCode.Stfld); il.Token(fieldReference(catalog.CellValue(capture.Capture.Type))); return;
        }
        if (target is BoundArrayIndex index) { EmitArrayStore(index, value); return; }
        if (target is BoundProperty property)
        {
            maxStack = Math.Max(maxStack, (property.Arguments?.Count ?? 0) + (property.Receiver is null ? 0 : 1) + ExpressionStack.Depth(value));
            EmitCall(property.Property.Setter!, (property.Arguments ?? []).Append(value).ToArray(), property.Receiver);
            return;
        }
        var field = (BoundField)target;
        if (field.Receiver is not null)
        {
            if (field.Receiver.Type.IsValue) EmitAddress(field.Receiver); else EmitReferenceReceiver(field.Receiver);
        }
        EmitExpression(value);
        il.OpCode(field.Field.IsStatic ? ILOpCode.Stsfld : ILOpCode.Stfld); il.Token(fieldReference(field.Field));
    }

    private void EmitHasValue(BoundExpression operand)
    {
        if (operand.Type.NullableValue) EmitCall(NullableMethod(operand.Type, "get_HasValue"), [], operand);
        else { EmitExpression(operand); if (operand.Type.IsParameter) { il.OpCode(ILOpCode.Box); il.Token(typeHandle(operand.Type)); } il.OpCode(ILOpCode.Ldnull); il.OpCode(ILOpCode.Cgt_un); }
    }

    private void EmitCoalesce(BoundCoalesce coalesce)
    {
        var slot = Temporary(coalesce.Left.Type == TypeSymbol.Null ? coalesce.Type with { IsNullable = true } : coalesce.Left.Type);
        EmitExpression(coalesce.Left); il.StoreLocal(slot);
        var local = new BoundLocal(new("<coalesce>", localTypes[slot], slot));
        var fallback = il.DefineLabel(); var done = il.DefineLabel();
        EmitHasValue(local); il.Branch(ILOpCode.Brfalse, fallback);
        if (local.Type.NullableValue)
        {
            var unwrapped = new BoundCall(NullableMethod(local.Type, "GetValueOrDefault"), [], local);
            EmitConversion(new(unwrapped, coalesce.Type, true));
        }
        else EmitConversion(new(local with { FlowType = local.Type.NonNullable }, coalesce.Type, true));
        il.Branch(ILOpCode.Br, done);
        il.MarkLabel(fallback); EmitExpression(coalesce.Right);
        il.MarkLabel(done);
    }

    private void EmitConversion(BoundConversion conversion)
    {
        var from = conversion.Operand.Type; var to = conversion.Target;
        if (from == TypeSymbol.Null)
        {
            EmitExpression(conversion.Operand);
            if (to.NullableValue) { il.OpCode(ILOpCode.Pop); EmitDefault(to); }
            return;
        }
        if (from.NullableValue && to.IsValue)
        {
            var operand = conversion.Operand;
            if (to.NullableValue)
            {
                var slot = Temporary(from); EmitExpression(operand); il.StoreLocal(slot);
                var local = new BoundLocal(new("<lift>", from, slot));
                var empty = il.DefineLabel(); var done = il.DefineLabel();
                EmitHasValue(local); il.Branch(ILOpCode.Brfalse, empty);
                EmitConversion(new(new BoundCall(NullableMethod(from, "GetValueOrDefault"), [], local), to.NonNullable, conversion.Checked));
                il.OpCode(ILOpCode.Newobj); il.Token(reference(NullableMethod(to, ".ctor", [to.NonNullable])));
                il.Branch(ILOpCode.Br, done); il.MarkLabel(empty); EmitDefault(to); il.MarkLabel(done);
            }
            else EmitConversion(new(new BoundCall(NullableMethod(from, "get_Value"), [], operand), to, conversion.Checked));
            return;
        }
        if (to.NullableValue && from.IsValue)
        {
            EmitConversion(new(conversion.Operand, to.NonNullable, conversion.Checked));
            il.OpCode(ILOpCode.Newobj); il.Token(reference(NullableMethod(to, ".ctor", [to.NonNullable])));
            return;
        }
        if (from.IsParameter && !from.NullableValue)
        {
            EmitExpression(conversion.Operand);
            if (to.IsParameter && !to.NullableValue && from.Parameter == to.Parameter) return;
            il.OpCode(ILOpCode.Box); il.Token(typeHandle(from));
            if (to.IsParameter || to.IsValue) { il.OpCode(ILOpCode.Unbox_any); il.Token(typeHandle(to)); }
            else if (!catalog.CanAssign(from, to)) { il.OpCode(ILOpCode.Castclass); il.Token(typeHandle(to)); }
            return;
        }
        if (to.IsParameter && !to.NullableValue)
        {
            EmitExpression(conversion.Operand);
            if (from.IsValue) { il.OpCode(ILOpCode.Box); il.Token(typeHandle(from)); }
            il.OpCode(ILOpCode.Unbox_any); il.Token(typeHandle(to)); return;
        }
        if (catalog.ScalarInfo(from) is { Category: not Syntax.ScalarCategory.Enum } domain &&
            (TypeFacts.Numeric(to) || to == domain.Base))
        {
            EmitConversion(new(new BoundCall(ReferenceCatalog.DomainGetter(domain), [], conversion.Operand), to, conversion.Checked)); return;
        }
        if (catalog.ScalarInfo(to) is { Category: not Syntax.ScalarCategory.Enum } targetDomain && from == targetDomain.Base)
        {
            EmitExpression(conversion.Operand); il.OpCode(ILOpCode.Newobj); il.Token(reference(ReferenceCatalog.DomainConstructor(targetDomain))); return;
        }
        EmitExpression(conversion.Operand);
        if (from == to || from.NonNullable == to.NonNullable) return;
        if (from.IsValue && to.IsReference) { il.OpCode(ILOpCode.Box); il.Token(typeHandle(from)); return; }
        if (from.IsReference && to.IsValue) { il.OpCode(ILOpCode.Unbox_any); il.Token(typeHandle(to)); return; }
        if (from.IsReference && to.IsReference)
        {
            if (!catalog.Derives(from, to)) { il.OpCode(ILOpCode.Castclass); il.Token(typeHandle(to)); }
            return;
        }
        if (catalog.ScalarInfo(from) is { Category: Syntax.ScalarCategory.Enum } fromScalar) from = fromScalar.Base;
        if (catalog.ScalarInfo(to) is { Category: Syntax.ScalarCategory.Enum } toEnum) to = toEnum.Base;
        EmitNumericConversion(from, to, conversion.Checked);
    }
}
