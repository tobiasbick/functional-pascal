using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed record ImportedEvent(ImportedMethod? Add, ImportedMethod? Remove);

internal sealed partial class ReferenceCatalog
{
    internal ImportedEvent? Event(TypeSymbol owner, string name, SourceText source, int position)
    {
        var declarations = new List<(TypeSymbol Owner, ImportedEvent? Event)>();
        foreach (var type in Hierarchy(owner))
        {
            if (type.Record is { } record)
            {
                if (record.Fields.Any(field => field.Name == name) || record.Methods.Any(method => method.Name == name))
                    declarations.Add((type, null));
                continue;
            }
            if (type.Union is not null || type.Kind == TypeKind.Scalar) continue;
            var definition = Definition(type); var reader = definition.Assembly.Reader;
            var found = false;
            foreach (var handle in definition.Definition.GetEvents())
            {
                var member = reader.GetEventDefinition(handle);
                if (reader.GetString(member.Name) != name) continue;
                var raw = member.Type.Kind switch
                {
                    HandleKind.TypeDefinition => FromDefinition(definition.Assembly, (TypeDefinitionHandle)member.Type),
                    HandleKind.TypeReference => FromReference(definition.Assembly, (TypeReferenceHandle)member.Type),
                    HandleKind.TypeSpecification => reader.GetTypeSpecification((TypeSpecificationHandle)member.Type).DecodeSignature(new SignatureTypes(this, definition.Assembly), null),
                    _ => TypeSymbol.Unsupported
                };
                var handler = Substitute(NullabilityMetadata.Member(raw, definition, member.GetCustomAttributes()), type);
                ImportedMethod? Accessor(MethodDefinitionHandle method)
                {
                    if (method.IsNil || (reader.GetMethodDefinition(method).Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Public) return null;
                    var imported = ConstrainStatic(ReadMethod(type, method), owner);
                    return imported is { Result.Kind: TypeKind.Void, Parameters.Count: 1, MethodArguments: null } &&
                        !imported.Parameter(0).ByRef && imported.Parameters[0].Erased == handler.Erased && DelegateSignature(handler) is not null ? imported : null;
                }
                var accessors = member.GetAccessors();
                declarations.Add((type, new(Accessor(accessors.Adder), Accessor(accessors.Remover))));
                found = true;
            }
            if (!found && (definition.Definition.GetFields().Any(handle => reader.GetString(reader.GetFieldDefinition(handle).Name) == name) ||
                definition.Definition.GetProperties().Any(handle => reader.GetString(reader.GetPropertyDefinition(handle).Name) == name) ||
                definition.Definition.GetMethods().Any(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == name)))
                declarations.Add((type, null));
        }
        var visible = declarations.Where(candidate => !declarations.Any(other => other.Owner != candidate.Owner &&
            Hierarchy(other.Owner).Skip(1).Any(parent => parent.Erased == candidate.Owner.Erased))).ToArray();
        if (visible.Length > 1 && visible.Any(candidate => candidate.Event is not null))
            throw source.Error("FP3034", $"Ambiguous inherited event '{owner}.{name}'; select a declaring interface explicitly.", position);
        return visible.Length == 1 ? visible[0].Event : null;
    }
}
