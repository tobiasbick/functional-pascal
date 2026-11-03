using Fpas.Compiler.Binding.Types;
using System.Collections.Immutable;
using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal void DiagnoseUnsupportedMethod(TypeSymbol owner, string name, bool isStatic, SourceText source, int position, int? argumentCount = null)
    {
        foreach (var type in Hierarchy(owner).Where(type => type.Definition is not null))
        {
            var definition = Definition(type);
            var reader = definition.Assembly.Reader;
            foreach (var handle in definition.Definition.GetMethods())
            {
                var method = reader.GetMethodDefinition(handle);
                if (reader.GetString(method.Name) != name || method.Attributes.HasFlag(MethodAttributes.Static) != isStatic ||
                    (method.Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Public) continue;
                if (isStatic && method.Attributes.HasFlag(MethodAttributes.Virtual) && !owner.IsParameter)
                    throw source.Error("FP3005", "Static virtual interface members require a constrained type parameter.", position);
                var limits = new SignatureLimits(this, definition.Assembly);
                var signature = method.DecodeSignature(limits, null);
                if (argumentCount is not null && signature.ParameterTypes.Length != argumentCount) continue;
                if (signature.ReturnType.Kind == TypeKind.ByRef) limits.Reasons.Add("ref returns");
                if (signature.Header.CallingConvention != SignatureCallingConvention.Default) limits.Reasons.Add("non-managed-default calling conventions");
                limits.Diagnose(source, position, $"{type}.{name}");
            }
        }
    }

    internal void DiagnoseUnsupportedMember(TypeSymbol owner, string name, bool isStatic, bool writing, SourceText source, int position)
    {
        foreach (var type in Hierarchy(owner).Where(type => type.Definition is not null))
        {
            var definition = Definition(type);
            var reader = definition.Assembly.Reader;
            foreach (var handle in definition.Definition.GetProperties())
            {
                var property = reader.GetPropertyDefinition(handle);
                if (reader.GetString(property.Name) != name) continue;
                var accessors = property.GetAccessors();
                var accessor = writing ? accessors.Setter : accessors.Getter;
                if (!accessor.IsNil) DiagnoseUnsupportedMethod(type, reader.GetString(reader.GetMethodDefinition(accessor).Name), isStatic, source, position);
            }
            foreach (var handle in definition.Definition.GetFields())
            {
                var field = reader.GetFieldDefinition(handle);
                if (reader.GetString(field.Name) != name || field.Attributes.HasFlag(FieldAttributes.Static) != isStatic ||
                    (field.Attributes & FieldAttributes.FieldAccessMask) != FieldAttributes.Public) continue;
                var limits = new SignatureLimits(this, definition.Assembly);
                field.DecodeSignature(limits, null);
                limits.Diagnose(source, position, $"{type}.{name}");
            }
        }
    }

    private sealed class SignatureLimits(ReferenceCatalog catalog, ReferenceAssembly assembly) : ISignatureTypeProvider<TypeSymbol, object?>
    {
        private readonly SignatureTypes types = new(catalog, assembly);
        internal HashSet<string> Reasons { get; } = new(StringComparer.Ordinal);
        internal void Diagnose(SourceText source, int position, string member)
        {
            if (Reasons.Count != 0)
                throw source.Error("FP3005", $"CLR member '{member}' requires unsupported features: {string.Join(", ", Reasons.Order(StringComparer.Ordinal))}.", position);
        }
        private TypeSymbol Reject(string reason) { Reasons.Add(reason); return TypeSymbol.Unsupported; }
        public TypeSymbol GetPrimitiveType(PrimitiveTypeCode code)
            => types.GetPrimitiveType(code) is { Kind: not TypeKind.Unsupported } type ? type : Reject($"CLR primitive '{code}'");
        public TypeSymbol GetGenericInstantiation(TypeSymbol type, ImmutableArray<TypeSymbol> arguments) => types.GetGenericInstantiation(type, arguments);
        public TypeSymbol GetGenericTypeParameter(object? context, int index) => types.GetGenericTypeParameter(context, index);
        public TypeSymbol GetGenericMethodParameter(object? context, int index) => types.GetGenericMethodParameter(context, index);
        public TypeSymbol GetByReferenceType(TypeSymbol type) => types.GetByReferenceType(type);
        public TypeSymbol GetArrayType(TypeSymbol type, ArrayShape shape)
            => shape.Rank == 1 ? Reject("non-vector rank-one CLR arrays") : types.GetArrayType(type, shape);
        public TypeSymbol GetSZArrayType(TypeSymbol type) => types.GetSZArrayType(type);
        public TypeSymbol GetTypeFromDefinition(MetadataReader reader, TypeDefinitionHandle handle, byte kind)
        {
            var definition = reader.GetTypeDefinition(handle);
            return Observe(types.GetTypeFromDefinition(reader, handle, kind),
                new ImportedType(assembly, handle, reader.GetString(definition.Namespace), reader.GetString(definition.Name)).FullName);
        }
        public TypeSymbol GetTypeFromReference(MetadataReader reader, TypeReferenceHandle handle, byte kind)
        {
            var reference = reader.GetTypeReference(handle);
            var ns = reader.GetString(reference.Namespace); var name = reader.GetString(reference.Name);
            return Observe(types.GetTypeFromReference(reader, handle, kind), ns.Length == 0 ? name : ns + "." + name);
        }
        public TypeSymbol GetTypeFromSpecification(MetadataReader reader, object? context, TypeSpecificationHandle handle, byte kind)
            => reader.GetTypeSpecification(handle).DecodeSignature(this, context);
        private TypeSymbol Observe(TypeSymbol type, string name)
        {
            if (type != TypeSymbol.Unsupported) return type;
            var byrefLike = catalog.types.TryGetValue(name, out var definitions) && definitions.Any(definition =>
                MetadataAttributes.Has(definition.Assembly.Reader, definition.Definition.GetCustomAttributes(), "System.Runtime.CompilerServices.IsByRefLikeAttribute"));
            return Reject(byrefLike ? $"byref-like type '{name}'" : $"inaccessible or unresolved CLR type '{name}'");
        }
        public TypeSymbol GetModifiedType(TypeSymbol modifier, TypeSymbol type, bool required)
        {
            var result = types.GetModifiedType(modifier, type, required);
            if (required && result == TypeSymbol.Unsupported)
                Reasons.Add(modifier.Definition?.FullName == "System.Runtime.CompilerServices.IsExternalInit"
                    ? "init-only setters" : $"required CLR modifier '{modifier}'");
            return result;
        }
        public TypeSymbol GetFunctionPointerType(MethodSignature<TypeSymbol> signature) => Reject("function pointers");
        public TypeSymbol GetPointerType(TypeSymbol type) => Reject("pointers");
        public TypeSymbol GetPinnedType(TypeSymbol type) => Reject("pinned signature types");
    }
}
