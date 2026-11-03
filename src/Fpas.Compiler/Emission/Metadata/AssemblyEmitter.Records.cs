using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private void EmitRecords(BoundCompilation program, MethodBodyStreamEncoder bodies, ref int parameterCount, TypeReferenceHandle objectType)
    {
        var fieldCount = metadata.GetRowCount(TableIndex.Field);
        foreach (var record in layout.Records)
        {
            genericTypeContext = record.GenericParameters.Count == 0 ? null : record.GenericParameters;
            var name = record.Name + (record.UnionCase is null && record.GenericParameters.Count != 0 ? "`" + record.GenericParameters.Count : "");
            var typeFlags = SourceTypeFlags(record.Attributes) | (record.UnionCase is { } serializableCase ? SourceTypeFlags(serializableCase.Owner.Attributes) : 0);
            var type = metadata.AddTypeDefinition(typeFlags | TypeAttributes.Sealed | (record.IsPublic || record.UnionCase is not null ? TypeAttributes.NestedPublic : TypeAttributes.NestedPrivate),
                default, metadata.GetOrAddString(name), record.UnionCase is { } member ? TypeHandle(member.Owner.Type) : objectType, MetadataTokens.FieldDefinitionHandle(fieldCount + 1), layout.SourceMethods[record.Constructor]);
            RegisterGenerics(type, record.GenericParameters, genericTypeContext);
            metadata.AddNestedType(type, layout.RecordOwners[record]);
            AddNullabilityContext(type, program.Runtime);
            AddSourceAttributes(type, record.Attributes);
            var equatable = catalog.Symbol(catalog.Required("System.IEquatable`1")) with { Arguments = new([record.Type]) };
            metadata.AddInterfaceImplementation(type, TypeHandle(equatable));
            foreach (var contract in record.Interfaces.Where(contract => contract.Erased != equatable.Erased))
                metadata.AddInterfaceImplementation(type, TypeHandle(contract));
            if (record.UnionCase is { } unionCase) AddUnionCaseAttribute(type, unionCase);
            else AddRecordAttribute(type);
            foreach (var field in record.Fields)
            {
                var signature = new BlobBuilder();
                EncodeType(new BlobEncoder(signature).FieldSignature(), field.Type);
                var handle = metadata.AddFieldDefinition(FieldAttributes.Private | FieldAttributes.InitOnly,
                    metadata.GetOrAddString("<" + field.Name + ">k__BackingField"), metadata.GetOrAddBlob(signature));
                fieldCount++;
                if (NullableFlags(field.Type).Contains((byte)2)) AddNullableAnnotation(handle, program.Runtime, field.Type);
            }
            foreach (var method in record.Methods)
            {
                var firstParameter = AddParameters(method, ref parameterCount, program.Runtime);
                var attributes = MethodAttributes.HideBySig | (method.IsPublic ? MethodAttributes.Public : MethodAttributes.Private);
                if (record.UnionCase is not null && method.Generated == GeneratedMethodKind.Constructor) attributes = attributes & ~MethodAttributes.MemberAccessMask | MethodAttributes.Assembly;
                if (!method.IsInstance) attributes |= MethodAttributes.Static;
                if (method.Generated is GeneratedMethodKind.Constructor) attributes |= MethodAttributes.SpecialName | MethodAttributes.RTSpecialName;
                if (method.Generated is GeneratedMethodKind.Getter or GeneratedMethodKind.Equality or GeneratedMethodKind.Inequality or GeneratedMethodKind.Tag) attributes |= MethodAttributes.SpecialName;
                if (method.Generated is GeneratedMethodKind.EqualsTyped) attributes |= MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot;
                if (method.Generated is GeneratedMethodKind.Tag) attributes |= MethodAttributes.Virtual;
                if (method.Generated is GeneratedMethodKind.EqualsObject or GeneratedMethodKind.Hash) attributes |= MethodAttributes.Virtual;
                if (record.InterfaceMethods.Any(implementation => implementation.Body == method) && method.Generated is GeneratedMethodKind.None or GeneratedMethodKind.Getter)
                    attributes |= MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot;
                var offset = method.Generated == GeneratedMethodKind.Constructor ? EmitRecordConstructor(record, bodies) : EmitBody(record.Bodies[method], bodies);
                var methodHandle = metadata.AddMethodDefinition(attributes, SourceMethodFlags(record.Bodies.GetValueOrDefault(method)?.Attributes), metadata.GetOrAddString(method.Name),
                    MethodSignature(method.Result, SourceSignatureParameters(method), method.IsInstance, genericArity: method.GenericParameters.Count), offset, firstParameter);
                RegisterGenerics(methodHandle, method.GenericParameters, genericTypeContext);
                if (method.IsAsync) AddAsyncAttribute(methodHandle, layout.AsyncRoutines[record.Bodies[method]]);
                AddSourceAttributes(methodHandle, record.Bodies.GetValueOrDefault(method)?.Attributes);
            }
            foreach (var implementation in record.InterfaceMethods.DistinctBy(implementation => (implementation.Contract.Owner, implementation.Contract.DefinitionHandle, implementation.Contract.SourceMethod)))
            {
                var contract = implementation.Contract;
                metadata.AddMethodImplementation(type, layout.SourceMethods[implementation.Body], Reference(contract, instantiate: false));
            }
            PropertyDefinitionHandle firstProperty = default;
            for (var index = 0; index < record.Fields.Count; index++)
            {
                var field = record.Fields[index];
                var signature = new BlobBuilder();
                new BlobEncoder(signature).PropertySignature(isInstanceProperty: true).Parameters(0,
                    result => EncodeType(result.Type(), field.Type), _ => { });
                var property = metadata.AddProperty(PropertyAttributes.None, metadata.GetOrAddString(field.Name), metadata.GetOrAddBlob(signature));
                if (firstProperty.IsNil) firstProperty = property;
                metadata.AddMethodSemantics(property, MethodSemanticsAttributes.Getter, layout.SourceMethods[field.Getter]);
                if (NullableFlags(field.Type).Contains((byte)2)) AddNullableAnnotation(property, program.Runtime, field.Type);
                if (record.UnionCase is null) AddRecordFieldAttribute(property, field, index);
                AddSourceAttributes(property, field.Attributes);
            }
            if (record.UnionCase is not null)
            {
                var tag = AddReadOnlyProperty("Tag", TypeSymbol.Int32, true, layout.SourceMethods[record.Methods.Single(method => method.Generated == GeneratedMethodKind.Tag)], program.Runtime);
                if (firstProperty.IsNil) firstProperty = tag;
            }
            if (!firstProperty.IsNil) metadata.AddPropertyMap(type, firstProperty);
        }
        genericTypeContext = null;
    }

    private int EmitRecordConstructor(RecordSymbol record, MethodBodyStreamEncoder bodies)
    {
        var il = new InstructionEncoder(new BlobBuilder());
        il.LoadArgument(0);
        il.Call(Reference(record.UnionCase is { } member ? member.Owner.Import(member.Owner.Method(GeneratedMethodKind.Constructor)) : new ImportedMethod(TypeSymbol.Object, ".ctor", TypeSymbol.Void, [], false)));
        for (var index = 0; index < record.Fields.Count; index++)
        {
            il.LoadArgument(0); il.LoadArgument(index + 1);
            il.OpCode(ILOpCode.Stfld); il.Token(FieldReference(catalog.RecordField(record.Fields[index], record.Type)));
        }
        il.OpCode(ILOpCode.Ret);
        return bodies.AddMethodBody(il, maxStack: 2);
    }

    private void AddRecordAttribute(TypeDefinitionHandle type)
    {
        var owner = catalog.Symbol(catalog.Required("Fpas.Core.RecordAttribute"));
        var constructor = new ImportedMethod(owner, ".ctor", TypeSymbol.Void, [TypeSymbol.Int32], false);
        var value = new BlobBuilder(); value.WriteUInt16(1); value.WriteInt32(1); value.WriteUInt16(0);
        metadata.AddCustomAttribute(type, Reference(constructor), metadata.GetOrAddBlob(value));
    }

    private void AddRecordFieldAttribute(PropertyDefinitionHandle property, RecordFieldSymbol field, int index)
    {
        var owner = catalog.Symbol(catalog.Required("Fpas.Core.RecordFieldAttribute"));
        var constructor = new ImportedMethod(owner, ".ctor", TypeSymbol.Void, [TypeSymbol.Int32, TypeSymbol.String, TypeSymbol.String with { IsNullable = true }], false);
        var value = new BlobBuilder(); value.WriteUInt16(1); value.WriteInt32(index);
        value.WriteSerializedString(field.Copy!.Name); value.WriteSerializedString(field.Default?.Name); value.WriteUInt16(0);
        metadata.AddCustomAttribute(property, Reference(constructor), metadata.GetOrAddBlob(value));
    }
}
