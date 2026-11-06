using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private TypeReferenceHandle DefinitionHandle(ImportedType definition)
    {
        if (types.TryGetValue(definition, out var handle)) return handle;
        EntityHandle scope = definition.DeclaringType is { } parent ? DefinitionHandle(parent) : Reference(definition.Assembly.Identity);
        handle = metadata.AddTypeReference(scope, metadata.GetOrAddString(definition.Namespace), metadata.GetOrAddString(definition.Name));
        types.Add(definition, handle);
        return handle;
    }

    private EntityHandle TypeHandle(TypeSymbol type)
    {
        type = type.Erased;
        if (type.Container is { } container) return layout.Containers[container.Syntax.Name.Text];
        if (type.Scalar is { } scalar && !type.NullableValue) return layout.ScalarTypes[scalar];
        if (type.Kind == TypeKind.Union && type.Arguments is not { Count: > 0 }) return layout.UnionTypes[type.Union!];
        if (type.Kind == TypeKind.Record && type.Arguments is not { Count: > 0 }) return layout.RecordTypes[type.Record!];
        if (type.Kind == TypeKind.Interface && type.Arguments is not { Count: > 0 }) return layout.InterfaceTypes[type.Interface!];
        if (type.Kind == TypeKind.Generated && type.Closure!.GenericParameters.Count == 0) return layout.ClosureTypes[type.Closure];
        if (type.Kind == TypeKind.Async && type.Arguments is not { Count: > 0 }) return layout.AsyncTypes[type.Machine!];
        var cacheKey = (type, genericTypeContext);
        if (typeHandles.TryGetValue(cacheKey, out var handle)) return handle;
        if (type.NullableValue || type.Kind is TypeKind.Array or TypeKind.Parameter or TypeKind.Generated or TypeKind.Async || type.Arguments is { Count: > 0 })
        {
            var signature = new BlobBuilder();
            EncodeType(new BlobEncoder(signature).TypeSpecificationSignature(), type);
            handle = metadata.AddTypeSpecification(metadata.GetOrAddBlob(signature));
        }
        else handle = DefinitionHandle(catalog.Definition(type));
        typeHandles.Add(cacheKey, handle);
        return handle;
    }

    private EntityHandle Reference(ImportedMethod method) => Reference(method, instantiate: true);

    private EntityHandle Reference(ImportedMethod method, bool instantiate)
    {
        EntityHandle handle;
        if (method.SourceMethod is { } source && ReferenceCatalog.SourceParameters(method.Owner).Count == 0)
            handle = layout.SourceMethods[source];
        else
        {
            if (method.SourceMethod is { } definition)
            {
                var parameters = ReferenceCatalog.SourceParameters(method.Owner);
                method = method with { SignatureResult = catalog.SourceSignature(definition.Result, parameters),
                    SignatureParameters = SourceSignatureParameters(definition).Select(parameter => catalog.SourceSignature(parameter, parameters)).ToArray() };
            }
            var signature = MethodSignature(method.SignatureResult ?? method.Result, method.SignatureParameters ?? method.DelegateParameters, !method.IsStatic, method.ReturnsByRef, method.MethodArguments?.Count ?? 0);
            var key = $"{MetadataTokens.GetToken(TypeHandle(method.Owner))}:{method.Name}:{MetadataTokens.GetHeapOffset(signature)}";
            if (!methods.TryGetValue(key, out var member))
            {
                member = metadata.AddMemberReference(TypeHandle(method.Owner), metadata.GetOrAddString(method.Name), signature);
                methods.Add(key, member);
            }
            handle = member;
        }
        if (method.MethodArguments is null || !instantiate) return handle;
        var arguments = new BlobBuilder();
        var encoder = new BlobEncoder(arguments).MethodSpecificationSignature(method.MethodArguments.Count);
        foreach (var argument in method.MethodArguments) EncodeType(encoder.AddArgument(), argument);
        var blob = metadata.GetOrAddBlob(arguments);
        var specKey = $"{MetadataTokens.GetToken(handle)}:{MetadataTokens.GetHeapOffset(blob)}";
        if (!methodSpecs.TryGetValue(specKey, out var specification))
        {
            specification = metadata.AddMethodSpecification(handle, blob);
            methodSpecs.Add(specKey, specification);
        }
        return specification;
    }

    private MemberReferenceHandle FieldReference(ImportedField field)
    {
        var signature = new BlobBuilder();
        EncodeType(new BlobEncoder(signature).FieldSignature(), field.SignatureType ?? field.Type);
        var blob = metadata.GetOrAddBlob(signature);
        var owner = TypeHandle(field.Owner);
        var key = $"{MetadataTokens.GetToken(owner)}:{field.Name}:{MetadataTokens.GetHeapOffset(blob)}";
        if (fields.TryGetValue(key, out var handle)) return handle;
        handle = metadata.AddMemberReference(owner, metadata.GetOrAddString(field.Name), blob);
        fields.Add(key, handle);
        return handle;
    }

    private BlobHandle MethodSignature(TypeSymbol resultType, IReadOnlyList<TypeSymbol> parameterTypes, bool instance = false, bool byRefResult = false, int genericArity = 0)
    {
        var signature = new BlobBuilder();
        new BlobEncoder(signature).MethodSignature(genericParameterCount: genericArity, isInstanceMethod: instance).Parameters(parameterTypes.Count,
            result => { if (resultType == TypeSymbol.Void) result.Void(); else EncodeType(result.Type(byRefResult), resultType); },
            parameters =>
            {
                foreach (var type in parameterTypes)
                {
                    var parameter = parameters.AddParameter();
                    if (type.Kind == TypeKind.ByRef && type.Definition is not null)
                        parameter.CustomModifiers().AddModifier(DefinitionHandle(type.Definition), isOptional: type.ParameterIndex == 0);
                    EncodeType(parameter.Type(isByRef: type.Kind == TypeKind.ByRef), type.Kind == TypeKind.ByRef ? type.Element! : type);
                }
            });
        return metadata.GetOrAddBlob(signature);
    }

    private IReadOnlyList<TypeSymbol> SourceSignatureParameters(RoutineSymbol method)
    {
        var contract = method.Owner?.InterfaceMethods.FirstOrDefault(implementation => implementation.Body == method).Contract;
        return method.Parameters.Select((parameter, index) => contract?.SignatureParameters?[index] is { Kind: TypeKind.ByRef, Definition: not null } modified
            ? parameter.SignatureType with { Definition = modified.Definition, ParameterIndex = modified.ParameterIndex }
            : parameter.Mode == "in" && (method.InterfaceOwner is not null || contract?.SourceMethod?.InterfaceOwner is not null)
                ? parameter.SignatureType with { Definition = catalog.Required("System.Runtime.InteropServices.InAttribute"), ParameterIndex = 1 }
                : parameter.SignatureType).ToArray();
    }

    private void EncodeType(SignatureTypeEncoder encoder, TypeSymbol type)
    {
        if (type.Kind == TypeKind.Generated && type.Closure!.GenericParameters.Count != 0)
        {
            var arguments = encoder.GenericInstantiation(layout.ClosureTypes[type.Closure], type.Closure.GenericParameters.Count, false);
            foreach (var argument in type.Arguments ?? new TypeList(type.Closure.GenericParameters.Select(parameter => parameter.Type))) EncodeType(arguments.AddArgument(), argument);
            return;
        }
        if (type.NullableValue)
        {
            var definition = catalog.Required("System.Nullable`1");
            var generic = DefinitionHandle(definition);
            EncodeType(encoder.GenericInstantiation(generic, 1, isValueType: true).AddArgument(), type.NonNullable);
            return;
        }
        if (type.Arguments is { Count: > 0 })
        {
            EntityHandle definition = type.Interface is { } contract ? layout.InterfaceTypes[contract] : type.Record is { } record ? layout.RecordTypes[record] : type.Union is { } union ? layout.UnionTypes[union] : type.Machine is { } machine ? layout.AsyncTypes[machine] : DefinitionHandle(type.Definition!);
            var arguments = encoder.GenericInstantiation(definition, type.Arguments.Count, type.IsValue);
            foreach (var argument in type.Arguments) EncodeType(arguments.AddArgument(), argument);
            return;
        }
        switch (type.Kind)
        {
            case TypeKind.String: encoder.String(); break;
            case TypeKind.Object: encoder.Object(); break;
            case TypeKind.Int32: encoder.Int32(); break;
            case TypeKind.Boolean: encoder.Boolean(); break;
            case TypeKind.Double: encoder.Double(); break;
            case TypeKind.SByte: encoder.SByte(); break;
            case TypeKind.Byte: encoder.Byte(); break;
            case TypeKind.Int16: encoder.Int16(); break;
            case TypeKind.UInt16: encoder.UInt16(); break;
            case TypeKind.UInt32: encoder.UInt32(); break;
            case TypeKind.Int64: encoder.Int64(); break;
            case TypeKind.UInt64: encoder.UInt64(); break;
            case TypeKind.NativeUInt: encoder.UIntPtr(); break;
            case TypeKind.Single: encoder.Single(); break;
            case TypeKind.Decimal: encoder.Type(DefinitionHandle(catalog.Definition(type)), isValueType: true); break;
            case TypeKind.Scalar: encoder.Type(layout.ScalarTypes[type.Scalar!], type.IsValue); break;
            case TypeKind.Char: encoder.Char(); break;
            case TypeKind.Array:
                if (type.Rank == 1) EncodeType(encoder.SZArray(), type.Element!);
                else encoder.Array(element => EncodeType(element, type.Element!), shape => shape.Shape(type.Rank, [], [.. Enumerable.Repeat(0, type.Rank)]));
                break;
            case TypeKind.Named: encoder.Type(DefinitionHandle(type.Definition!), type.IsValue); break;
            case TypeKind.Parameter:
                var slot = genericTypeContext?.ToList().IndexOf(type.Parameter!) ?? -1;
                if (slot >= 0) encoder.GenericTypeParameter(slot); else encoder.GenericMethodTypeParameter(type.Parameter!.Index);
                break;
            case TypeKind.TypeParameter: encoder.GenericTypeParameter(type.ParameterIndex); break;
            case TypeKind.MethodParameter: encoder.GenericMethodTypeParameter(type.ParameterIndex); break;
            case TypeKind.NativeInt: encoder.IntPtr(); break;
            case TypeKind.Generated:
            case TypeKind.Union:
            case TypeKind.Record: encoder.Type(TypeHandle(type), isValueType: false); break;
            case TypeKind.Interface: encoder.Type(TypeHandle(type), isValueType: false); break;
            case TypeKind.Async: encoder.Type(TypeHandle(type), isValueType: false); break;
            default: throw new InvalidOperationException("Unsupported emitted type.");
        }
    }

    private static IEnumerable<byte> NullableFlags(TypeSymbol type)
    {
        if (type.IsParameter && type.NullableValue)
        {
            yield return 0;
            yield break;
        }
        if (type.CanHaveNull || type.IsParameter || type.Arguments is { Count: > 0 }) yield return type.CanHaveNull || type.IsParameter ? (byte)(type.IsNullable ? 2 : 1) : (byte)0;
        if (type.Element is not null) foreach (var flag in NullableFlags(type.Element)) yield return flag;
        if (type.Arguments is not null) foreach (var argument in type.Arguments) foreach (var flag in NullableFlags(argument)) yield return flag;
    }

    private void AddNullableAnnotation(EntityHandle parameter, AssemblyIdentity runtime, TypeSymbol type)
    {
        AddNullableFlags(parameter, runtime, NullableFlags(type).ToArray());
    }

    private void AddNullableFlags(EntityHandle parameter, AssemblyIdentity runtime, IReadOnlyList<byte> flags)
    {
        if (nullableAnnotationConstructor.IsNil)
        {
            var attribute = metadata.AddTypeReference(Reference(runtime), metadata.GetOrAddString("System.Runtime.CompilerServices"), metadata.GetOrAddString("NullableAttribute"));
            var signature = new BlobBuilder();
            new BlobEncoder(signature).MethodSignature(isInstanceMethod: true).Parameters(1, result => result.Void(), args => args.AddParameter().Type().SZArray().Byte());
            nullableAnnotationConstructor = metadata.AddMemberReference(attribute, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(signature));
        }
        var value = new BlobBuilder();
        value.WriteUInt16(1); value.WriteInt32(flags.Count);
        foreach (var flag in flags) value.WriteByte(flag);
        value.WriteUInt16(0);
        metadata.AddCustomAttribute(parameter, nullableAnnotationConstructor, metadata.GetOrAddBlob(value));
    }
}
