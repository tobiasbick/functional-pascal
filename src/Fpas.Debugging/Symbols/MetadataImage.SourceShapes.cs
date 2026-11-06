using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Debugging.Symbols;

internal sealed record SourceMember(string Name, uint Field);
internal sealed record SourceShape(string Kind, SourceMember[] Members, string? Primitive = null, string? Lower = null, string? Upper = null,
    int? Tag = null, string? Case = null);

internal partial class MetadataImage
{
    internal SourceShape? Shape(uint token)
    {
        var definition = Metadata.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.Handle((int)token));
        var attributes = definition.GetCustomAttributes();
        if (Contract(attributes, "DomainAttribute", [0x20, 4, 1, 8, 14, 14, 14]) is { } domain)
        {
            if (domain.ReadInt32() != 1) throw new BadImageFormatException("Unsupported FPAS domain version.");
            var primitive = ContractName(ref domain); var lower = domain.ReadSerializedString(); var upper = domain.ReadSerializedString(); ContractEnd(ref domain);
            if (lower is null != (upper is null) || lower?.Length > 128 || upper?.Length > 128) throw new BadImageFormatException("Invalid FPAS domain bounds.");
            return new("domain", [new("Value", BackingField(token, "<Value>"))], primitive, lower, upper);
        }
        if (Contract(attributes, "UnionCaseAttribute", [0x20, 4, 1, 8, 14, 14, 0x1d, 14]) is { } union)
        {
            var parent = definition.GetDeclaringType();
            if (parent.IsNil || Contract(Metadata.GetTypeDefinition(parent).GetCustomAttributes(), "UnionAttribute", [0x20, 1, 1, 8]) is not { } marker
                || marker.ReadInt32() != 1) throw new BadImageFormatException("Invalid FPAS union owner.");
            ContractEnd(ref marker);
            var tag = union.ReadInt32(); var name = ContractName(ref union); var factory = ContractName(ref union); var count = union.ReadInt32();
            if (tag < 0 || count is < 0 or > 4096 || factory != (count == 0 ? "get_" + name : name)) throw new BadImageFormatException("Invalid FPAS union case.");
            var members = new List<SourceMember>(); var names = new HashSet<string>(StringComparer.Ordinal);
            for (var index = 0; index < count; index++)
            {
                var label = ContractName(ref union);
                if (!names.Add(label)) throw new BadImageFormatException("Duplicate FPAS union field.");
                members.Add(new(label, BackingField(token, "<" + label + ">k__BackingField")));
            }
            ContractEnd(ref union); return new("union", members.ToArray(), Tag: tag, Case: name);
        }
        if (Contract(attributes, "RecordAttribute", [0x20, 1, 1, 8]) is not { } record) return null;
        if (record.ReadInt32() != 1) throw new BadImageFormatException("Unsupported FPAS record version.");
        ContractEnd(ref record); var fields = new SortedDictionary<int, SourceMember>();
        if (definition.GetProperties().Count > 4096) throw new BadImageFormatException("Too many FPAS record properties.");
        foreach (var propertyHandle in definition.GetProperties())
        {
            var property = Metadata.GetPropertyDefinition(propertyHandle);
            if (Contract(property.GetCustomAttributes(), "RecordFieldAttribute", [0x20, 3, 1, 8, 14, 14]) is not { } field) continue;
            var order = field.ReadInt32(); var copy = ContractName(ref field); var optional = field.ReadSerializedString(); ContractEnd(ref field);
            var name = Metadata.GetString(property.Name);
            if (order < 0 || fields.ContainsKey(order) || copy != "<With:" + name + ">" || optional is not null && optional != "<Default:" + name + ">") throw new BadImageFormatException("Invalid FPAS record field.");
            fields.Add(order, new(name, BackingField(token, "<" + name + ">k__BackingField")));
        }
        if (!fields.Keys.SequenceEqual(Enumerable.Range(0, fields.Count))) throw new BadImageFormatException("Invalid FPAS record field order.");
        return new("record", fields.Values.ToArray());
    }

    private uint BackingField(uint token, string name)
    {
        var fields = Fields(token).Take(4097).ToArray();
        if (fields.Length > 4096) throw new BadImageFormatException("Too many stored FPAS fields.");
        var matching = fields.Where(field => field.Name == name).ToArray();
        if (matching.Length != 1) throw new BadImageFormatException("Missing or ambiguous FPAS stored field.");
        var definition = Metadata.GetFieldDefinition((FieldDefinitionHandle)MetadataTokens.Handle((int)matching[0].Token));
        if ((definition.Attributes & (FieldAttributes.Static | FieldAttributes.InitOnly)) != FieldAttributes.InitOnly) throw new BadImageFormatException("Invalid FPAS stored field.");
        return matching[0].Token;
    }
}
