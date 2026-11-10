using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    private readonly Dictionary<TypeSymbol, RecordInfo?> records = [];

    internal RecordInfo? RecordInfo(TypeSymbol type)
    {
        type = type.NonNullable;
        if (type.Record is { } record) return record.VariantCase is null ? SourceRecordInfo(type) : null;
        if (type.Kind != TypeKind.Named) return null;
        if (records.TryGetValue(type, out var cached)) return cached;
        var definition = Definition(type);
        var marker = RecordAttribute(definition, definition.Definition.GetCustomAttributes(), "RecordAttribute");
        if (marker is null) { records.Add(type, null); return null; }
        var version = marker.Value;
        if (version.ReadInt32() != 1 || version.ReadUInt16() != 0 || version.RemainingBytes != 0 ||
            Decode(definition, definition.Definition.BaseType, type) != TypeSymbol.Object || !definition.Definition.Attributes.HasFlag(TypeAttributes.Sealed) || type.IsValue || type.TypeArguments.Count != definition.Definition.GetGenericParameters().Count)
            throw InvalidRecord(type);
        var fields = new SortedDictionary<int, RecordFieldInfo>();
        var names = new HashSet<string>(StringComparer.Ordinal);
        foreach (var handle in definition.Definition.GetProperties())
        {
            var property = definition.Assembly.Reader.GetPropertyDefinition(handle);
            var data = RecordAttribute(definition, property.GetCustomAttributes(), "RecordFieldAttribute");
            if (data is null) continue;
            var blob = data.Value;
            var order = blob.ReadInt32();
            var copyName = blob.ReadSerializedString(); var defaultName = blob.ReadSerializedString();
            var name = definition.Assembly.Reader.GetString(property.Name);
            var accessors = property.GetAccessors();
            if (blob.ReadUInt16() != 0 || blob.RemainingBytes != 0 || accessors.Getter.IsNil || !accessors.Setter.IsNil || copyName != "<With:" + name + ">" ||
                defaultName is not null && defaultName != "<Default:" + name + ">") throw InvalidRecord(type);
            var getter = ReadMethod(type, accessors.Getter, includePrivate: true) ?? throw InvalidRecord(type);
            var visibility = definition.Assembly.Reader.GetMethodDefinition(accessors.Getter).Attributes & MethodAttributes.MemberAccessMask;
            if (!names.Add(name) || definition.Assembly.Reader.GetMethodDefinition(accessors.Getter).GetDeclaringType() != definition.Handle || getter.IsStatic || getter.Parameters.Count != 0 || getter.Result == TypeSymbol.Void || visibility is not (MethodAttributes.Public or MethodAttributes.Private)) throw InvalidRecord(type);
            if (visibility == MethodAttributes.Public && !PublicType(getter.Result)) throw InvalidRecord(type);
            var signature = property.DecodeSignature(new SignatureTypes(this, definition.Assembly, includePrivate: true), null);
            if (signature.ParameterTypes.Length != 0 || Substitute(NullabilityMetadata.Member(signature.ReturnType, definition, property.GetCustomAttributes()), type) != getter.Result) throw InvalidRecord(type);
            var copy = RecordHelper(type, copyName, false, [getter.Result], type, visibility);
            var provider = defaultName is null ? null : RecordHelper(type, defaultName, true, [], getter.Result, null);
            if (order < 0 || !fields.TryAdd(order, new(name, getter.Result, visibility == MethodAttributes.Public, getter, copy, provider))) throw InvalidRecord(type);
        }
        if (!fields.Keys.SequenceEqual(Enumerable.Range(0, fields.Count))) throw InvalidRecord(type);
        var all = fields.Values.ToArray();
        var publicConstructor = all.All(field => field.IsPublic);
        var constructor = RecordHelper(type, ".ctor", false, all.Select(field => field.Type).ToArray(), TypeSymbol.Void,
            publicConstructor ? MethodAttributes.Public : MethodAttributes.Private);
        foreach (var field in all.Where(field => field.Default is not null))
        {
            var method = definition.Assembly.Reader.GetMethodDefinition(field.Default!.DefinitionHandle);
            if ((method.Attributes & MethodAttributes.MemberAccessMask) != (publicConstructor ? MethodAttributes.Public : MethodAttributes.Private)) throw InvalidRecord(type);
        }
        var result = new RecordInfo(type, all, constructor, publicConstructor);
        records.Add(type, result);
        return result;
    }

    private ImportedMethod RecordHelper(TypeSymbol owner, string name, bool isStatic, IReadOnlyList<TypeSymbol> parameters, TypeSymbol result, MethodAttributes? visibility)
    {
        var definition = Definition(owner);
        var matches = definition.Definition.GetMethods().Select(handle => ReadMethod(owner, handle, includePrivate: true)).Where(method => method is not null &&
            method.Name == name && method.IsStatic == isStatic && method.Parameters.SequenceEqual(parameters) && method.Result == result &&
            definition.Assembly.Reader.GetMethodDefinition(method.DefinitionHandle).GetGenericParameters().Count == 0).ToArray();
        if (matches.Length != 1) throw InvalidRecord(owner);
        var match = matches[0]!;
        if (visibility is not null && (definition.Assembly.Reader.GetMethodDefinition(match.DefinitionHandle).Attributes & MethodAttributes.MemberAccessMask) != visibility) throw InvalidRecord(owner);
        return match;
    }

    private static BlobReader? RecordAttribute(ImportedType owner, CustomAttributeHandleCollection attributes, string name)
        => MetadataAttributes.Contract(owner.Assembly.Reader, attributes, name, name == "RecordAttribute" ? [0x20, 1, 1, 8] : [0x20, 3, 1, 8, 14, 14]);

    private static bool PublicType(TypeSymbol type) => type.Definition?.IsPublic != false &&
        (type.Element is null || PublicType(type.Element)) && (type.Arguments is null || type.Arguments.All(PublicType));

    private static BadImageFormatException InvalidRecord(TypeSymbol type) => new($"Invalid or unsupported FPAS record metadata on '{type}'.");
}
