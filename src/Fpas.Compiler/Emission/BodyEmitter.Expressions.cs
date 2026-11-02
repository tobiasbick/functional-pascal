using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class BodyEmitter
{
    private void EmitExpression(BoundExpression expression)
    {
        switch (expression)
        {
            case BoundNoValue: break;
            case BoundPendingException:
                il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.MachineField(asyncMachine!, "<>exception", catalog.Symbol(catalog.Required("System.Exception"))))); break;
            case BoundAddress address: EmitAddress(address.Target, force: true); break;
            case BoundDereference dereference:
                EmitExpression(dereference.Address); il.OpCode(ILOpCode.Ldobj); il.Token(typeHandle(dereference.Type)); break;
            case BoundTypeTest test:
                EmitExpression(test.Value);
                if (test.Value.Type.IsValue || test.Value.Type.IsParameter) { il.OpCode(ILOpCode.Box); il.Token(typeHandle(test.Value.Type)); }
                il.OpCode(ILOpCode.Isinst); il.Token(typeHandle(test.Target)); il.OpCode(ILOpCode.Ldnull); il.OpCode(ILOpCode.Cgt_un);
                break;
            case BoundFailure failure:
                il.LoadString(userString(failure.Message)); il.OpCode(ILOpCode.Newobj);
                il.Token(reference(new ImportedMethod(catalog.Symbol(catalog.Required("System.InvalidOperationException")), ".ctor", TypeSymbol.Void, [TypeSymbol.String], false)));
                il.OpCode(ILOpCode.Throw);
                break;
            case BoundSequence sequence:
                foreach (var binding in sequence.Bindings) EmitStatement(binding);
                EmitExpression(sequence.Value);
                break;
            case BoundRecordField field:
                EmitExpression(field.Receiver); il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.RecordField(field.Field, field.Receiver.Type)));
                break;
            case BoundSafeCast cast:
                EmitExpression(cast.Value); il.OpCode(ILOpCode.Isinst); il.Token(typeHandle(cast.Target));
                break;
            case BoundNewArray array: EmitNewArray(array); break;
            case BoundArrayLiteral array: EmitArrayLiteral(array); break;
            case BoundArrayIndex index: EmitArrayIndex(index); break;
            case BoundArrayLength length:
                EmitExpression(length.Receiver); il.OpCode(ILOpCode.Ldlen); il.OpCode(ILOpCode.Conv_ovf_i4_un); break;
            case BoundCharacter character: il.LoadConstantI4(character.Value); break;
            case BoundDelegate callable: EmitDelegate(callable); break;
            case BoundCapture capture: EmitCapture(capture); break;
            case BoundCaptureStorage storage: EmitCaptureStorage(storage.Capture); break;
            case BoundLocalStorage storage: LoadStorage(storage.Local); break;
            case BoundNull:
                il.OpCode(ILOpCode.Ldnull);
                break;
            case BoundNew creation:
                foreach (var argument in creation.Arguments) EmitExpression(argument);
                if (creation.Constructor is null) EmitDefault(creation.Type);
                else { il.OpCode(ILOpCode.Newobj); il.Token(reference(creation.Constructor)); }
                break;
            case BoundProperty property:
                EmitCall(property.Property.Getter!, property.Arguments ?? [], property.Receiver);
                break;
            case BoundField field:
                EmitField(field);
                break;
            case BoundCoalesce coalesce:
                EmitCoalesce(coalesce);
                break;
            case BoundNullTest test:
                EmitHasValue(test.Operand);
                if (test.EqualsNull) Invert();
                break;
            case BoundString text:
                il.LoadString(userString(text.Value));
                break;
            case BoundInteger number:
                il.LoadConstantI4(number.Value);
                break;
            case BoundDouble number:
                il.LoadConstantR8(number.Value);
                break;
            case BoundBoolean boolean:
                il.LoadConstantI4(boolean.Value ? 1 : 0);
                break;
            case BoundLocal local:
                LoadStorage(local.Local);
                if (local.Local.Captured && local.Local.Mutable) { il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.CellValue(local.Local.Type))); }
                break;
            case BoundParameter parameter:
                if (asyncMachine is not null) { il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(fieldReference(catalog.MachineParameter(asyncMachine, parameter.Parameter))); }
                else il.LoadArgument(parameter.Parameter.Index + argumentOffset);
                if (parameter.Parameter.ByRef) { il.OpCode(ILOpCode.Ldobj); il.Token(typeHandle(parameter.Parameter.Type)); }
                break;
            case BoundCall call:
                EmitCall(call.Method, call.Arguments, call.Receiver);
                break;
            case BoundConversion conversion:
                EmitConversion(conversion);
                break;
            case BoundUnary unary:
                var checkedNegation = unary.Operator == "-" && unary.Checked && unary.Type == TypeSymbol.Int32;
                if (checkedNegation)
                    il.LoadConstantI4(0);
                EmitExpression(unary.Operand);
                if (unary.Operator == "-")
                    il.OpCode(checkedNegation ? ILOpCode.Sub_ovf : ILOpCode.Neg);
                if (unary.Operator == "not")
                {
                    if (unary.Type == TypeSymbol.Boolean)
                        Invert();
                    else
                        il.OpCode(ILOpCode.Not);
                }
                break;
            case BoundBinary binary:
                EmitBinary(binary);
                break;
            case BoundConditional conditional:
                var done = il.DefineLabel();
                foreach (var arm in conditional.Arms)
                {
                    var next = il.DefineLabel();
                    EmitExpression(arm.Condition);
                    il.Branch(ILOpCode.Brfalse, next);
                    EmitExpression(arm.Value);
                    il.Branch(ILOpCode.Br, done);
                    il.MarkLabel(next);
                }
                EmitExpression(conditional.Else);
                il.MarkLabel(done);
                break;
            default:
                throw new InvalidOperationException("Unrecognized bound expression.");
        }
    }

    private void EmitBinary(BoundBinary binary)
    {
        if (binary.Left.Type == TypeSymbol.Boolean && binary.Operator is "and" or "or")
        {
            var shortCircuit = il.DefineLabel();
            var done = il.DefineLabel();
            EmitExpression(binary.Left);
            il.Branch(binary.Operator == "and" ? ILOpCode.Brfalse : ILOpCode.Brtrue, shortCircuit);
            EmitExpression(binary.Right);
            il.Branch(ILOpCode.Br, done);
            il.MarkLabel(shortCircuit);
            il.LoadConstantI4(binary.Operator == "and" ? 0 : 1);
            il.MarkLabel(done);
            return;
        }
        EmitExpression(binary.Left);
        EmitExpression(binary.Right);
        var check = binary.Checked && binary.Left.Type == TypeSymbol.Int32;
        var real = binary.Left.Type == TypeSymbol.Double;
        il.OpCode(binary.Operator switch
        {
            "+" => check ? ILOpCode.Add_ovf : ILOpCode.Add,
            "-" => check ? ILOpCode.Sub_ovf : ILOpCode.Sub,
            "*" => check ? ILOpCode.Mul_ovf : ILOpCode.Mul,
            "/" or "div" => ILOpCode.Div, "mod" => ILOpCode.Rem,
            "and" => ILOpCode.And, "or" => ILOpCode.Or, "xor" => ILOpCode.Xor,
            "shl" => ILOpCode.Shl, "shr" => ILOpCode.Shr,
            "=" or "<>" => ILOpCode.Ceq, "<" => ILOpCode.Clt, ">" => ILOpCode.Cgt,
            "<=" => real ? ILOpCode.Cgt_un : ILOpCode.Cgt,
            ">=" => real ? ILOpCode.Clt_un : ILOpCode.Clt,
            _ => throw new InvalidOperationException("Unrecognized bound operator.")
        });
        if (binary.Operator is "<>" or "<=" or ">=")
            Invert();
    }

    private void Invert()
    {
        il.LoadConstantI4(0);
        il.OpCode(ILOpCode.Ceq);
    }
}
