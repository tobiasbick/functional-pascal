using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using System.Globalization;
using System.Numerics;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private void EmitScalars(MethodBodyStreamEncoder bodies, ref int parameterCount, AssemblyIdentity runtime)
    {
        foreach (var scalar in layout.ScalarTypes.Keys)
        {
            var info = catalog.ScalarInfo(scalar.Type)!;
            var firstField = MetadataTokens.FieldDefinitionHandle(metadata.GetRowCount(TableIndex.Field) + 1);
            var firstMethod = MetadataTokens.MethodDefinitionHandle(metadata.GetRowCount(TableIndex.MethodDef) + 1);
            var parent = catalog.Symbol(catalog.Required(info.Category == ScalarCategory.Enum ? "System.Enum" : info.Type.IsValue ? "System.ValueType" : "System.Object"));
            var attributes = TypeAttributes.Sealed | (scalar.IsPublic ? TypeAttributes.NestedPublic : TypeAttributes.NestedPrivate);
            if (info.Type.IsValue && info.Category != ScalarCategory.Enum) attributes |= TypeAttributes.SequentialLayout;
            var handle = metadata.AddTypeDefinition(attributes | SourceTypeFlags(scalar.Attributes), default, metadata.GetOrAddString(scalar.Name), TypeHandle(parent), firstField, firstMethod);
            metadata.AddNestedType(handle, layout.ScalarOwners[scalar]); AddNullabilityContext(handle, runtime);
            AddSourceAttributes(handle, scalar.Attributes);
            if (info.Category == ScalarCategory.Enum)
            {
                ScalarField("value__", info.Base, FieldAttributes.Public | FieldAttributes.SpecialName | FieldAttributes.RTSpecialName);
                foreach (var member in scalar.Members)
                    metadata.AddConstant(ScalarField(member.Name.Text, info.Type, FieldAttributes.Public | FieldAttributes.Static | FieldAttributes.Literal | FieldAttributes.HasDefault), member.Value);
                continue;
            }
            var storage = ScalarField("<Value>", info.Category == ScalarCategory.Range ? TypeSymbol.UInt64 : info.Base, FieldAttributes.Private | FieldAttributes.InitOnly);
            AddGeneratedField(storage);
            AddDebuggerDisplay(handle, scalar.Name + "({Value})", scalar.Attributes);
            if (info.Category == ScalarCategory.Range)
            {
                metadata.AddConstant(ScalarField("LowerBound", info.Base, FieldAttributes.Public | FieldAttributes.Static | FieldAttributes.Literal | FieldAttributes.HasDefault), NumericValue.ConvertTo(info.Lower!.Value, info.Base, true));
                metadata.AddConstant(ScalarField("UpperBound", info.Base, FieldAttributes.Public | FieldAttributes.Static | FieldAttributes.Literal | FieldAttributes.HasDefault), NumericValue.ConvertTo(info.Upper!.Value, info.Base, true));
            }
            var equatable = catalog.Symbol(catalog.Required("System.IEquatable`1")) with { Arguments = new([info.Type]) };
            metadata.AddInterfaceImplementation(handle, TypeHandle(equatable));
            AddDomainAttribute(handle, info);
            ScalarMethod(".ctor", TypeSymbol.Void, [("Value", info.Base)], false, MethodAttributes.SpecialName | MethodAttributes.RTSpecialName,
                il => ScalarConstructor(il, info, storage), bodies, ref parameterCount, runtime);
            var getter = ScalarMethod("get_Value", info.Base, [], false, MethodAttributes.SpecialName,
                il => ScalarValue(il, info, storage), bodies, ref parameterCount, runtime);
            metadata.AddPropertyMap(handle, AddReadOnlyProperty("Value", info.Base, true, getter, runtime));
            ScalarMethod("op_Explicit", info.Type, [("Value", info.Base)], true, MethodAttributes.SpecialName,
                il => { il.LoadArgument(0); il.OpCode(ILOpCode.Newobj); il.Token(Reference(ReferenceCatalog.DomainConstructor(info))); il.OpCode(ILOpCode.Ret); }, bodies, ref parameterCount, runtime);
            ScalarMethod(info.Category == ScalarCategory.Range ? "op_Implicit" : "op_Explicit", info.Base, [("Value", info.Type)], true, MethodAttributes.SpecialName,
                il => { LoadDomainValue(il, info, 0); il.OpCode(ILOpCode.Ret); }, bodies, ref parameterCount, runtime);
            EmitScalarEquality(info, bodies, ref parameterCount, runtime);
        }
    }

    private FieldDefinitionHandle ScalarField(string name, TypeSymbol type, FieldAttributes attributes)
    {
        var blob = new BlobBuilder(); EncodeType(new BlobEncoder(blob).FieldSignature(), type);
        return metadata.AddFieldDefinition(attributes, metadata.GetOrAddString(name), metadata.GetOrAddBlob(blob));
    }

    private MethodDefinitionHandle ScalarMethod(string name, TypeSymbol result, IReadOnlyList<(string Name, TypeSymbol Type)> parameters,
        bool isStatic, MethodAttributes attributes, Action<InstructionEncoder> emit, MethodBodyStreamEncoder bodies, ref int parameterCount, AssemblyIdentity runtime)
    {
        var first = MetadataTokens.ParameterHandle(parameterCount + 1);
        foreach (var parameter in parameters)
        {
            var handle = metadata.AddParameter(ParameterAttributes.None, metadata.GetOrAddString(parameter.Name), ++parameterCount - MetadataTokens.GetRowNumber(first) + 1);
            if (NullableFlags(parameter.Type).Contains((byte)2)) AddNullableAnnotation(handle, runtime, parameter.Type);
        }
        var il = new InstructionEncoder(new BlobBuilder(), new ControlFlowBuilder()); emit(il);
        var method = metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.HideBySig | attributes | (isStatic ? MethodAttributes.Static : 0), MethodImplAttributes.IL,
            metadata.GetOrAddString(name), MethodSignature(result, parameters.Select(parameter => parameter.Type).ToArray(), !isStatic), bodies.AddMethodBody(il, maxStack: 8), first);
        AddGeneratedMethod(method);
        return method;
    }

    private void AddDomainAttribute(TypeDefinitionHandle handle, ScalarInfo info)
    {
        var type = catalog.Symbol(catalog.Required("Fpas.Core.DomainAttribute"));
        var constructor = new ImportedMethod(type, ".ctor", TypeSymbol.Void, [TypeSymbol.Int32, TypeSymbol.String, TypeSymbol.String with { IsNullable = true }, TypeSymbol.String with { IsNullable = true }], false);
        var blob = new BlobBuilder(); blob.WriteUInt16(1); blob.WriteInt32(1); blob.WriteSerializedString(catalog.Definition(info.Base).FullName);
        blob.WriteSerializedString(info.Lower?.ToString(CultureInfo.InvariantCulture)); blob.WriteSerializedString(info.Upper?.ToString(CultureInfo.InvariantCulture)); blob.WriteUInt16(0);
        metadata.AddCustomAttribute(handle, Reference(constructor), metadata.GetOrAddBlob(blob));
    }

    private void LoadDomainValue(InstructionEncoder il, ScalarInfo info, int argument, bool self = false)
    {
        if (info.Type.IsValue && !self) il.LoadArgumentAddress(argument); else il.LoadArgument(argument);
        il.Call(Reference(ReferenceCatalog.DomainGetter(info)));
    }

    private static void IntegralConstant(InstructionEncoder il, BigInteger number, TypeSymbol type)
    {
        if (NumericFacts.Bits(type) == 64) il.LoadConstantI8(unchecked((long)(ulong)(number & ulong.MaxValue)));
        else il.LoadConstantI4(unchecked((int)(uint)(number & uint.MaxValue)));
    }

    private static ILOpCode IntegralNarrow(TypeSymbol type) => type.Kind switch
    {
        TypeKind.SByte => ILOpCode.Conv_i1, TypeKind.Byte => ILOpCode.Conv_u1, TypeKind.Int16 => ILOpCode.Conv_i2,
        TypeKind.UInt16 => ILOpCode.Conv_u2, TypeKind.UInt32 => ILOpCode.Conv_u4, _ => ILOpCode.Conv_i4
    };

    private void ScalarConstructor(InstructionEncoder il, ScalarInfo info, FieldDefinitionHandle field)
    {
        if (info.Type.IsReference)
        {
            il.LoadArgument(0); il.Call(Reference(new ImportedMethod(TypeSymbol.Object, ".ctor", TypeSymbol.Void, [], false)));
            var valid = il.DefineLabel(); il.LoadArgument(1); il.Branch(ILOpCode.Brtrue, valid);
            ScalarThrow(il, "System.ArgumentNullException"); il.MarkLabel(valid);
        }
        if (info.Category == ScalarCategory.Range)
        {
            var invalid = il.DefineLabel(); var valid = il.DefineLabel();
            il.LoadArgument(1); IntegralConstant(il, info.Lower!.Value, info.Base); il.OpCode(NumericFacts.Unsigned(info.Base) ? ILOpCode.Clt_un : ILOpCode.Clt); il.Branch(ILOpCode.Brtrue, invalid);
            il.LoadArgument(1); IntegralConstant(il, info.Upper!.Value, info.Base); il.OpCode(NumericFacts.Unsigned(info.Base) ? ILOpCode.Cgt_un : ILOpCode.Cgt); il.Branch(ILOpCode.Brfalse, valid);
            il.MarkLabel(invalid); ScalarThrow(il, "System.ArgumentOutOfRangeException"); il.MarkLabel(valid);
        }
        il.LoadArgument(0); il.LoadArgument(1);
        if (info.Category == ScalarCategory.Range)
        {
            il.OpCode(NumericFacts.Unsigned(info.Base) ? ILOpCode.Conv_u8 : ILOpCode.Conv_i8);
            IntegralConstant(il, info.Lower!.Value, TypeSymbol.UInt64); il.OpCode(ILOpCode.Sub);
        }
        il.OpCode(ILOpCode.Stfld); il.Token(field); il.OpCode(ILOpCode.Ret);
    }

    private void ScalarValue(InstructionEncoder il, ScalarInfo info, FieldDefinitionHandle field)
    {
        il.LoadArgument(0); il.OpCode(ILOpCode.Ldfld); il.Token(field);
        if (info.Category == ScalarCategory.Range)
        {
            IntegralConstant(il, info.Lower!.Value, TypeSymbol.UInt64); il.OpCode(ILOpCode.Add);
            if (NumericFacts.Bits(info.Base) < 64) il.OpCode(IntegralNarrow(info.Base));
        }
        il.OpCode(ILOpCode.Ret);
    }

    private void ScalarThrow(InstructionEncoder il, string name)
    {
        il.LoadString(metadata.GetOrAddUserString("Value")); il.OpCode(ILOpCode.Newobj);
        il.Token(Reference(new ImportedMethod(catalog.Symbol(catalog.Required(name)), ".ctor", TypeSymbol.Void, [TypeSymbol.String], false))); il.OpCode(ILOpCode.Throw);
    }
}
