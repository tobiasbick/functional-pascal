using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private void EmitScalarEquality(ScalarInfo info, MethodBodyStreamEncoder bodies, ref int parameterCount, AssemblyIdentity runtime)
    {
        var nullable = info.Type.IsReference ? info.Type with { IsNullable = true } : info.Type;
        var typedEquals = new ImportedMethod(info.Type, "Equals", TypeSymbol.Boolean, [nullable], false, true);
        var comparer = catalog.Symbol(catalog.Required("System.Collections.Generic.EqualityComparer`1")) with { Arguments = new([info.Base]) };
        var instance = catalog.Property(comparer, "Default", true)!.Getter!;
        var equals = catalog.Methods(comparer, "Equals", false).Single(method => method.Parameters.Count == 2);
        var hash = catalog.Methods(comparer, "GetHashCode", false).Single(method => method.Parameters.Count == 1);
        ScalarMethod("Equals", TypeSymbol.Boolean, [("Other", nullable)], false, MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot, il =>
        {
            var empty = il.DefineLabel();
            if (info.Type.IsReference) { il.LoadArgument(1); il.Branch(ILOpCode.Brfalse, empty); }
            il.Call(Reference(instance)); LoadDomainValue(il, info, 0, self: true); LoadDomainValue(il, info, 1);
            il.OpCode(ILOpCode.Callvirt); il.Token(Reference(equals)); il.OpCode(ILOpCode.Ret);
            if (info.Type.IsReference) { il.MarkLabel(empty); il.LoadConstantI4(0); il.OpCode(ILOpCode.Ret); }
        }, bodies, ref parameterCount, runtime);
        ScalarMethod("Equals", TypeSymbol.Boolean, [("Other", TypeSymbol.Object with { IsNullable = true })], false, MethodAttributes.Virtual, il =>
        {
            var different = il.DefineLabel(); il.LoadArgument(1); il.OpCode(ILOpCode.Isinst); il.Token(TypeHandle(info.Type)); il.Branch(ILOpCode.Brfalse, different);
            il.LoadArgument(0); il.LoadArgument(1); il.OpCode(info.Type.IsValue ? ILOpCode.Unbox_any : ILOpCode.Castclass); il.Token(TypeHandle(info.Type));
            il.Call(Reference(typedEquals)); il.OpCode(ILOpCode.Ret); il.MarkLabel(different); il.LoadConstantI4(0); il.OpCode(ILOpCode.Ret);
        }, bodies, ref parameterCount, runtime);
        ScalarMethod("GetHashCode", TypeSymbol.Int32, [], false, MethodAttributes.Virtual, il =>
        {
            il.Call(Reference(instance)); LoadDomainValue(il, info, 0, self: true); il.OpCode(ILOpCode.Callvirt); il.Token(Reference(hash)); il.OpCode(ILOpCode.Ret);
        }, bodies, ref parameterCount, runtime);
        ScalarMethod("op_Equality", TypeSymbol.Boolean, [("Left", nullable), ("Right", nullable)], true, MethodAttributes.SpecialName, il =>
        {
            if (info.Type.IsReference)
            {
                var same = il.DefineLabel(); var different = il.DefineLabel();
                il.LoadArgument(0); il.LoadArgument(1); il.OpCode(ILOpCode.Ceq); il.Branch(ILOpCode.Brtrue, same);
                il.LoadArgument(0); il.Branch(ILOpCode.Brfalse, different);
                il.LoadArgument(0); il.LoadArgument(1); il.Call(Reference(typedEquals)); il.OpCode(ILOpCode.Ret);
                il.MarkLabel(same); il.LoadConstantI4(1); il.OpCode(ILOpCode.Ret); il.MarkLabel(different); il.LoadConstantI4(0); il.OpCode(ILOpCode.Ret);
            }
            else { il.LoadArgumentAddress(0); il.LoadArgument(1); il.Call(Reference(typedEquals)); il.OpCode(ILOpCode.Ret); }
        }, bodies, ref parameterCount, runtime);
        ScalarMethod("op_Inequality", TypeSymbol.Boolean, [("Left", nullable), ("Right", nullable)], true, MethodAttributes.SpecialName, il =>
        {
            il.LoadArgument(0); il.LoadArgument(1); il.Call(Reference(new ImportedMethod(info.Type, "op_Equality", TypeSymbol.Boolean, [nullable, nullable], true)));
            il.LoadConstantI4(0); il.OpCode(ILOpCode.Ceq); il.OpCode(ILOpCode.Ret);
        }, bodies, ref parameterCount, runtime);
        EmitScalarOrdering(info, bodies, ref parameterCount, runtime);
    }

    private void EmitScalarOrdering(ScalarInfo info, MethodBodyStreamEncoder bodies, ref int count, AssemblyIdentity runtime)
    {
        if (!TypeFacts.Numeric(info.Base) && info.Base != TypeSymbol.String) return;
        foreach (var (name, op) in new[] { ("op_LessThan", "<"), ("op_GreaterThan", ">"), ("op_LessThanOrEqual", "<="), ("op_GreaterThanOrEqual", ">=") })
            ScalarMethod(name, TypeSymbol.Boolean, [("Left", info.Type), ("Right", info.Type)], true, MethodAttributes.SpecialName, il =>
            {
                LoadDomainValue(il, info, 0); LoadDomainValue(il, info, 1);
                if (info.Base == TypeSymbol.Decimal)
                    il.Call(Reference(new ImportedMethod(TypeSymbol.Decimal, name, TypeSymbol.Boolean, [TypeSymbol.Decimal, TypeSymbol.Decimal], true)));
                else
                {
                    if (info.Base == TypeSymbol.String)
                    { il.Call(Reference(new ImportedMethod(TypeSymbol.String, "CompareOrdinal", TypeSymbol.Int32, [TypeSymbol.String, TypeSymbol.String], true))); il.LoadConstantI4(0); }
                    var unsigned = NumericFacts.Unsigned(info.Base) || NumericFacts.Floating(info.Base) && op is "<=" or ">=";
                    il.OpCode(op is "<" or ">=" ? unsigned ? ILOpCode.Clt_un : ILOpCode.Clt : unsigned ? ILOpCode.Cgt_un : ILOpCode.Cgt);
                    if (op is "<=" or ">=") { il.LoadConstantI4(0); il.OpCode(ILOpCode.Ceq); }
                }
                il.OpCode(ILOpCode.Ret);
            }, bodies, ref count, runtime);
    }
}
