using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.References;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Emission.Bodies;

internal sealed partial class BodyEmitter
{
    private void EmitNumber(object value)
    {
        switch (value)
        {
            case decimal number:
                var bits = decimal.GetBits(number);
                il.LoadConstantI4(bits[0]); il.LoadConstantI4(bits[1]); il.LoadConstantI4(bits[2]);
                il.LoadConstantI4(bits[3] < 0 ? 1 : 0); il.LoadConstantI4((bits[3] >> 16) & 255);
                il.OpCode(ILOpCode.Newobj); il.Token(reference(new(TypeSymbol.Decimal, ".ctor", TypeSymbol.Void,
                    [TypeSymbol.Int32, TypeSymbol.Int32, TypeSymbol.Int32, TypeSymbol.Boolean, TypeSymbol.Byte], false)));
                maxStack = Math.Max(maxStack, 5); break;
            case float number: il.LoadConstantR4(number); break;
            case double number: il.LoadConstantR8(number); break;
            case long number: il.LoadConstantI8(number); break;
            case ulong number: il.LoadConstantI8(unchecked((long)number)); break;
            case nint number: il.LoadConstantI8((long)number); il.OpCode(ILOpCode.Conv_i); break;
            case nuint number: il.LoadConstantI8(unchecked((long)(ulong)number)); il.OpCode(ILOpCode.Conv_u); break;
            case uint number: il.LoadConstantI4(unchecked((int)number)); break;
            case char number: il.LoadConstantI4(number); break;
            case bool boolean: il.LoadConstantI4(boolean ? 1 : 0); break;
            default: il.LoadConstantI4(System.Convert.ToInt32(value, System.Globalization.CultureInfo.InvariantCulture)); break;
        }
    }

    private void EmitNumericConversion(TypeSymbol from, TypeSymbol to, bool check)
    {
        if (from == to) return;
        if (from == TypeSymbol.Decimal || to == TypeSymbol.Decimal)
        {
            if (to == TypeSymbol.Decimal)
            {
                var parameter = from.Kind is TypeKind.SByte or TypeKind.Byte or TypeKind.Int16 or TypeKind.UInt16 or TypeKind.Char ? TypeSymbol.Int32 : from;
                if (from.Kind is TypeKind.NativeInt or TypeKind.NativeUInt)
                { parameter = NumericFacts.Unsigned(from) ? TypeSymbol.UInt64 : TypeSymbol.Int64; EmitNumericConversion(from, parameter, false); }
                il.Call(reference(new ImportedMethod(TypeSymbol.Decimal, NumericFacts.Floating(from) ? "op_Explicit" : "op_Implicit", to, [parameter], true)));
            }
            else
            {
                var result = to == TypeSymbol.Char ? TypeSymbol.UInt16 : to == TypeSymbol.NativeInt ? TypeSymbol.Int64 : to == TypeSymbol.NativeUInt ? TypeSymbol.UInt64 : to;
                il.Call(reference(new ImportedMethod(TypeSymbol.Decimal, "op_Explicit", result, [from], true)));
                if (result != to) EmitNumericConversion(result, to, check: true);
            }
            return;
        }
        if (NumericFacts.Floating(to))
        {
            if (NumericFacts.Unsigned(from)) il.OpCode(ILOpCode.Conv_r_un);
            il.OpCode(to == TypeSymbol.Single ? ILOpCode.Conv_r4 : ILOpCode.Conv_r8); return;
        }
        var unsigned = NumericFacts.Unsigned(from);
        il.OpCode((to.Kind, check, unsigned) switch
        {
            (TypeKind.SByte, true, true) => ILOpCode.Conv_ovf_i1_un, (TypeKind.SByte, true, false) => ILOpCode.Conv_ovf_i1,
            (TypeKind.Byte, true, true) => ILOpCode.Conv_ovf_u1_un, (TypeKind.Byte, true, false) => ILOpCode.Conv_ovf_u1,
            (TypeKind.Int16, true, true) => ILOpCode.Conv_ovf_i2_un, (TypeKind.Int16, true, false) => ILOpCode.Conv_ovf_i2,
            (TypeKind.UInt16 or TypeKind.Char, true, true) => ILOpCode.Conv_ovf_u2_un, (TypeKind.UInt16 or TypeKind.Char, true, false) => ILOpCode.Conv_ovf_u2,
            (TypeKind.Int32, true, true) => ILOpCode.Conv_ovf_i4_un, (TypeKind.Int32, true, false) => ILOpCode.Conv_ovf_i4,
            (TypeKind.UInt32, true, true) => ILOpCode.Conv_ovf_u4_un, (TypeKind.UInt32, true, false) => ILOpCode.Conv_ovf_u4,
            (TypeKind.Int64, true, true) => ILOpCode.Conv_ovf_i8_un, (TypeKind.Int64, true, false) => ILOpCode.Conv_ovf_i8,
            (TypeKind.UInt64, true, true) => ILOpCode.Conv_ovf_u8_un, (TypeKind.UInt64, true, false) => ILOpCode.Conv_ovf_u8,
            (TypeKind.NativeInt, true, true) => ILOpCode.Conv_ovf_i_un, (TypeKind.NativeInt, true, false) => ILOpCode.Conv_ovf_i,
            (TypeKind.NativeUInt, true, true) => ILOpCode.Conv_ovf_u_un, (TypeKind.NativeUInt, true, false) => ILOpCode.Conv_ovf_u,
            (TypeKind.SByte, _, _) => ILOpCode.Conv_i1, (TypeKind.Byte, _, _) => ILOpCode.Conv_u1,
            (TypeKind.Int16, _, _) => ILOpCode.Conv_i2, (TypeKind.UInt16 or TypeKind.Char, _, _) => ILOpCode.Conv_u2,
            (TypeKind.Int32, _, _) => ILOpCode.Conv_i4, (TypeKind.UInt32, _, _) => ILOpCode.Conv_u4,
            (TypeKind.Int64, _, _) => unsigned ? ILOpCode.Conv_u8 : ILOpCode.Conv_i8,
            (TypeKind.UInt64, _, _) => NumericFacts.Integral(from) && !unsigned ? ILOpCode.Conv_i8 : ILOpCode.Conv_u8,
            (TypeKind.NativeInt, _, _) => unsigned ? ILOpCode.Conv_u : ILOpCode.Conv_i,
            (TypeKind.NativeUInt, _, _) => NumericFacts.Integral(from) && !unsigned ? ILOpCode.Conv_i : ILOpCode.Conv_u,
            _ => throw new InvalidOperationException("Unknown numeric conversion.")
        });
    }
}
