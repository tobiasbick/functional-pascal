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
    private void EmitVariants(BoundCompilation program, MethodBodyStreamEncoder bodies, ref int parameterCount, TypeReferenceHandle objectType)
    {
        foreach (var variant in layout.Variants)
        {
            genericTypeContext = variant.GenericParameters.Count == 0 ? null : variant.GenericParameters;
            var type = metadata.AddTypeDefinition(SourceTypeFlags(variant.Attributes) | TypeAttributes.Abstract | (variant.IsPublic ? TypeAttributes.NestedPublic : TypeAttributes.NestedPrivate),
                default, metadata.GetOrAddString(variant.Name + (variant.GenericParameters.Count == 0 ? "" : "`" + variant.GenericParameters.Count)), objectType, MetadataTokens.FieldDefinitionHandle(metadata.GetRowCount(TableIndex.Field) + 1), layout.SourceMethods[variant.Method(GeneratedMethodKind.Constructor)]);
            RegisterGenerics(type, variant.GenericParameters, genericTypeContext);
            metadata.AddNestedType(type, layout.VariantOwners[variant]);
            AddNullabilityContext(type, program.Runtime);
            AddSourceAttributes(type, variant.Attributes);
            var equatable = catalog.Symbol(catalog.Required("System.IEquatable`1")) with { Arguments = new([variant.Type]) };
            metadata.AddInterfaceImplementation(type, TypeHandle(equatable));
            var marker = catalog.Symbol(catalog.Required("Fpas.Core.VariantAttribute"));
            var value = new BlobBuilder(); value.WriteUInt16(1); value.WriteInt32(1); value.WriteUInt16(0);
            metadata.AddCustomAttribute(type, Reference(new ImportedMethod(marker, ".ctor", TypeSymbol.Void, [TypeSymbol.Int32], false)), metadata.GetOrAddBlob(value));
            foreach (var method in variant.Methods)
            {
                var firstParameter = AddParameters(method, ref parameterCount, program.Runtime);
                var attributes = MethodAttributes.HideBySig | (method.IsPublic ? MethodAttributes.Public : MethodAttributes.FamANDAssem);
                var isAbstract = method.Generated is GeneratedMethodKind.Tag or GeneratedMethodKind.EqualsObject or GeneratedMethodKind.Hash;
                if (!method.IsInstance) attributes |= MethodAttributes.Static;
                if (method.Generated == GeneratedMethodKind.Constructor) attributes |= MethodAttributes.SpecialName | MethodAttributes.RTSpecialName;
                if (method.Generated is GeneratedMethodKind.Equality or GeneratedMethodKind.Inequality or GeneratedMethodKind.Tag || method.Generated == GeneratedMethodKind.Factory && method.Parameters.Count == 0) attributes |= MethodAttributes.SpecialName;
                if (isAbstract) attributes |= MethodAttributes.Abstract | MethodAttributes.Virtual;
                if (method.Generated is GeneratedMethodKind.Tag) attributes |= MethodAttributes.NewSlot;
                if (method.Generated is GeneratedMethodKind.EqualsTyped) attributes |= MethodAttributes.Virtual | MethodAttributes.NewSlot | MethodAttributes.Final;
                var offset = -1;
                if (!isAbstract)
                {
                    if (method.Generated == GeneratedMethodKind.Constructor)
                    {
                        var il = new InstructionEncoder(new BlobBuilder()); il.LoadArgument(0);
                        il.Call(Reference(new ImportedMethod(TypeSymbol.Object, ".ctor", TypeSymbol.Void, [], false))); il.OpCode(ILOpCode.Ret);
                        offset = bodies.AddMethodBody(il, maxStack: 1);
                    }
                    else offset = EmitBody(variant.Bodies[method], bodies);
                }
                var methodHandle = metadata.AddMethodDefinition(attributes, MethodImplAttributes.IL, metadata.GetOrAddString(method.Name),
                    MethodSignature(method.Result, method.Parameters.Select(parameter => parameter.Type).ToArray(), method.IsInstance), offset, firstParameter);
                AddGeneratedMethod(methodHandle);
            }
            var firstProperty = AddReadOnlyProperty("Tag", TypeSymbol.Int32, true, layout.SourceMethods[variant.Method(GeneratedMethodKind.Tag)], program.Runtime);
            foreach (var member in variant.Cases.Where(member => member.Syntax.Fields.Count == 0))
                AddReadOnlyProperty(member.Syntax.Name.Text, variant.Type, false, layout.SourceMethods[member.Factory], program.Runtime);
            metadata.AddPropertyMap(type, firstProperty);
        }
        genericTypeContext = null;
    }

    private PropertyDefinitionHandle AddReadOnlyProperty(string name, TypeSymbol type, bool instance, MethodDefinitionHandle getter, AssemblyIdentity runtime)
    {
        var signature = new BlobBuilder();
        new BlobEncoder(signature).PropertySignature(instance).Parameters(0, result => EncodeType(result.Type(), type), _ => { });
        var property = metadata.AddProperty(PropertyAttributes.None, metadata.GetOrAddString(name), metadata.GetOrAddBlob(signature));
        metadata.AddMethodSemantics(property, MethodSemanticsAttributes.Getter, getter);
        if (NullableFlags(type).Contains((byte)2)) AddNullableAnnotation(property, runtime, type);
        return property;
    }

    private void AddVariantCaseAttribute(TypeDefinitionHandle type, VariantCaseSymbol member)
    {
        var marker = catalog.Symbol(catalog.Required("Fpas.Core.VariantCaseAttribute"));
        var constructor = new ImportedMethod(marker, ".ctor", TypeSymbol.Void, [TypeSymbol.Int32, TypeSymbol.String, TypeSymbol.String, TypeSymbol.Array(TypeSymbol.String)], false);
        var value = new BlobBuilder(); value.WriteUInt16(1); value.WriteInt32(member.Tag);
        value.WriteSerializedString(member.Syntax.Name.Text); value.WriteSerializedString(member.Factory.Name);
        value.WriteInt32(member.Representation.Fields.Count);
        foreach (var field in member.Representation.Fields) value.WriteSerializedString(field.Name);
        value.WriteUInt16(0);
        metadata.AddCustomAttribute(type, Reference(constructor), metadata.GetOrAddBlob(value));
    }
}
