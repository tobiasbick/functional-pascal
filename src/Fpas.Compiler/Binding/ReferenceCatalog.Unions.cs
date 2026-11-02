using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    private readonly Dictionary<TypeSymbol, UnionInfo?> unions = [];

    internal UnionInfo? UnionInfo(TypeSymbol type)
    {
        type = type.NonNullable;
        if (type.Union is not null) return SourceUnionInfo(type);
        if (type.Kind != TypeKind.Named) return null;
        if (unions.TryGetValue(type, out var existing)) return existing;
        var owner = Definition(type); var reader = owner.Assembly.Reader;
        var marker = MetadataAttributes.Contract(reader, owner.Definition.GetCustomAttributes(), "UnionAttribute", [0x20, 1, 1, 8]);
        if (marker is null) { unions.Add(type, null); return null; }
        var blob = marker.Value;
        if (blob.ReadInt32() != 1 || blob.ReadUInt16() != 0 || blob.RemainingBytes != 0 ||
            !owner.Definition.Attributes.HasFlag(TypeAttributes.Abstract) || owner.Definition.Attributes.HasFlag(TypeAttributes.Sealed) ||
            owner.Definition.GetGenericParameters().Count != type.TypeArguments.Count || Decode(owner, owner.Definition.BaseType, type) != TypeSymbol.Object)
            throw InvalidUnion(type);
        var constructors = owner.Definition.GetMethods().Where(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == ".ctor").ToArray();
        if (constructors.Length != 1 || ReadMethod(type, constructors[0]) is not { Parameters.Count: 0, IsStatic: false } ||
            (reader.GetMethodDefinition(constructors[0]).Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.FamANDAssem) throw InvalidUnion(type);
        var tag = Property(type, "Tag", false);
        if (tag?.Type != TypeSymbol.Int32 || tag.Setter is not null || tag.Getter is null ||
            !reader.GetMethodDefinition(tag.Getter.DefinitionHandle).Attributes.HasFlag(MethodAttributes.Abstract)) throw InvalidUnion(type);
        var cases = new SortedDictionary<int, UnionCaseInfo>();
        var names = new HashSet<string>(["Tag", "get_Tag", "Equals", "GetHashCode", "op_Equality", "op_Inequality", owner.Name], StringComparer.Ordinal);
        foreach (var handle in owner.Definition.GetNestedTypes())
        {
            var definition = reader.GetTypeDefinition(handle);
            var caseType = FromDefinition(owner.Assembly, handle, includePrivate: true) with { Arguments = type.Arguments };
            var data = MetadataAttributes.Contract(reader, definition.GetCustomAttributes(), "UnionCaseAttribute", [0x20, 4, 1, 8, 14, 14, 0x1d, 14]);
            if (data is null || !definition.Attributes.HasFlag(TypeAttributes.Sealed) || definition.Attributes.HasFlag(TypeAttributes.Abstract) ||
                definition.GetGenericParameters().Count != type.TypeArguments.Count || Decode(Definition(caseType), definition.BaseType, caseType) != type ||
                (definition.Attributes & TypeAttributes.VisibilityMask) != TypeAttributes.NestedPublic) throw InvalidUnion(type);
            var value = data.Value; var index = value.ReadInt32(); var name = value.ReadSerializedString(); var factoryName = value.ReadSerializedString();
            var count = value.ReadInt32();
            if (name is null || factoryName != (count == 0 ? "get_" + name : name) || reader.GetString(definition.Name) != name + "Case" ||
                count < 0 || count > value.RemainingBytes || !names.Add(name) || !names.Add(name + "Case") || count == 0 && !names.Add("get_" + name)) throw InvalidUnion(type);
            var fields = new List<UnionPayload>(); var labels = new HashSet<string>(["Tag", "get_Tag", "Equals", "GetHashCode", "op_Equality", "op_Inequality", name + "Case"], StringComparer.Ordinal);
            for (var field = 0; field < count; field++)
            {
                var label = value.ReadSerializedString();
                if (label is null || !labels.Add(label) || !labels.Add("get_" + label)) throw InvalidUnion(type);
                var property = Property(caseType, label, false);
                if (property?.Getter is null || property.Setter is not null || property.Getter.Owner != caseType || !PublicType(property.Type)) throw InvalidUnion(type);
                fields.Add(new(label, property.Type, property.Getter));
            }
            if (value.ReadUInt16() != 0 || value.RemainingBytes != 0) throw InvalidUnion(type);
            var factories = Methods(type, factoryName, true).Where(method => method.Parameters.SequenceEqual(fields.Select(field => field.Type)) && method.Result == type).ToArray();
            var factory = factories.Length == 1 ? factories[0] : null;
            if (factory is null || fields.Count == 0 && Property(type, name, true)?.Getter?.DefinitionHandle != factory.DefinitionHandle) throw InvalidUnion(type);
            var parameterNames = reader.GetMethodDefinition(factory.DefinitionHandle).GetParameters().Select(reader.GetParameter).Where(parameter => parameter.SequenceNumber != 0)
                .OrderBy(parameter => parameter.SequenceNumber).Select(parameter => reader.GetString(parameter.Name));
            if (!parameterNames.SequenceEqual(fields.Select(field => field.Name))) throw InvalidUnion(type);
            var stored = definition.GetFields().Select(reader.GetFieldDefinition).ToArray();
            if (stored.Length != fields.Count || stored.Any(field => field.Attributes.HasFlag(FieldAttributes.Static) || !field.Attributes.HasFlag(FieldAttributes.InitOnly) ||
                (field.Attributes & FieldAttributes.FieldAccessMask) != FieldAttributes.Private)) throw InvalidUnion(type);
            var properties = definition.GetProperties().Select(reader.GetPropertyDefinition).ToArray();
            if (properties.Length != fields.Count + 1 || properties.Any(property => !property.GetAccessors().Setter.IsNil)) throw InvalidUnion(type);
            var concreteTag = Property(caseType, "Tag", false);
            if (concreteTag?.Type != TypeSymbol.Int32 || concreteTag.Getter?.Owner != caseType || !concreteTag.Getter.IsVirtual ||
                reader.GetMethodDefinition(concreteTag.Getter.DefinitionHandle).Attributes.HasFlag(MethodAttributes.Abstract)) throw InvalidUnion(type);
            var ctor = RecordHelper(caseType, ".ctor", false, fields.Select(field => field.Type).ToArray(), TypeSymbol.Void, MethodAttributes.Assembly);
            if (definition.GetMethods().Count(method => reader.GetString(reader.GetMethodDefinition(method).Name) == ".ctor") != 1 ||
                ctor.Parameters.Count != fields.Count || !cases.TryAdd(index, new(name, index, caseType, factory, fields))) throw InvalidUnion(type);
        }
        if (cases.Count == 0 || !cases.Keys.SequenceEqual(Enumerable.Range(0, cases.Count))) throw InvalidUnion(type);
        var result = new UnionInfo(type, cases.Values.ToArray()); unions.Add(type, result); return result;
    }

    private static BadImageFormatException InvalidUnion(TypeSymbol type) => new($"Invalid or unsupported FPAS union metadata on '{type}'.");
}
