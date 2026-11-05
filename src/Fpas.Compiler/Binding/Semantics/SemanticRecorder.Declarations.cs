using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    internal void Declarations(SourceCatalog catalog)
    {
        foreach (var unit in catalog.Units)
        {
            token.ThrowIfCancellationRequested();
            var source = unit.Source;
            foreach (var record in unit.RecordTypes) GenericParameters(record.GenericParameters, record.Syntax.TypeParameters ?? [], source, "!");
            foreach (var union in unit.Unions.Values) GenericParameters(union.GenericParameters, union.Syntax.TypeParameters ?? [], source, "!");
            foreach (var contract in unit.Interfaces.Values) GenericParameters(contract.GenericParameters, contract.Syntax.TypeParameters, source, "!");
            Declare(unit, "container", unit.Syntax.Name.Text, source, unit.Syntax.Name.Parts[^1], metadata: TypeMetadata(unit.ContainerType));
            foreach (var method in unit.Routines.Values.SelectMany(group => group)) Routine(method, source, unit.ContainerType);
            foreach (var global in unit.Globals.Values)
                Declare(global, "global", global.Name, source, global.Syntax.Binding.Name, global.Type, Metadata(unit.ContainerType, "field", global.Name));
            foreach (var alias in unit.Aliases.Values)
                Declare(alias, "alias", alias.Syntax.Name.Text, source, alias.Syntax.Name, alias.Target, Metadata(unit.ContainerType, "alias", alias.Syntax.Name.Text));
            foreach (var scalar in unit.Scalars.Values)
            {
                Declare(scalar, "domain", scalar.Name, source, scalar.Syntax.Name, scalar.Type, TypeMetadata(scalar.Type));
                foreach (var member in scalar.Members) Declare(member, "enum_member", member.Name.Text, source, member.Name, scalar.Type, Metadata(scalar.Type, "field", member.Name.Text));
            }
            foreach (var record in unit.Records.Values)
            {
                Declare(record, "record", record.Name, source, record.Syntax.Name, record.Type, TypeMetadata(record.Type));
                RecordFields(record, source);
                foreach (var method in record.UserMethods.Values.SelectMany(group => group)) Routine(method, source, record.Type);
            }
            foreach (var contract in unit.Interfaces.Values)
            {
                Declare(contract, "interface", contract.Name, source, contract.Syntax.Name, contract.Type, TypeMetadata(contract.Type));
                foreach (var property in contract.Properties)
                {
                    Declare(property, "property", property.Syntax.Name.Text, source, property.Syntax.Name, property.Getter.Result, Metadata(contract.Type, "property", property.Syntax.Name.Text));
                    symbols[property.Getter] = symbols[property];
                    Export(MethodMetadata(ReferenceCatalog.SourceMethod(property.Getter, contract.Type)), symbols[property]);
                }
                foreach (var method in contract.Methods.Where(method => method.Declaration is not null)) Routine(method, source, contract.Type);
            }
            foreach (var union in unit.Unions.Values)
            {
                Declare(union, "union", union.Name, source, union.Syntax.Name, union.Type, TypeMetadata(union.Type));
                foreach (var item in union.Cases)
                {
                    Declare(item, "union_case", item.Syntax.Name.Text, source, item.Syntax.Name, union.Type,
                        MethodMetadata(ReferenceCatalog.SourceMethod(item.Factory, union.Type)));
                    symbols[item.Factory] = symbols[item];
                    symbols[item.Representation] = symbols[item];
                    Export(TypeMetadata(item.Representation.Type), symbols[item]);
                    RecordFields(item.Representation, source);
                }
            }
        }
        foreach (var record in catalog.Units.SelectMany(unit => unit.Records.Values))
            foreach (var (contract, body) in record.InterfaceMethods)
                if (symbols.TryGetValue(body, out var implementation)) relations.Add((implementation, Method(contract)));
    }

    private void RecordFields(RecordSymbol record, SourceText source)
    {
        foreach (var field in record.Fields)
        {
            Declare(field, "property", field.Name, source, field.Syntax.Name, field.Type, Metadata(record.Type, "property", field.Name));
            symbols[field.Getter] = symbols[field];
            Export(MethodMetadata(ReferenceCatalog.SourceMethod(field.Getter, record.Type)), symbols[field]);
        }
    }

    internal void Routine(RoutineSymbol method, SourceText source, TypeSymbol? owner = null)
    {
        if (method.Declaration is not { } declaration) return;
        if (symbols.TryGetValue(method, out var existing) && existing.Declaration is not null) return;
        GenericParameters(method.GenericParameters, declaration.TypeParameters ?? [], source, "!!");
        var imported = owner is null ? null : ReferenceCatalog.SourceMethod(method, owner);
        Declare(method, "routine", method.Name, source, declaration.Name, metadata: imported is null ? null : MethodMetadata(imported),
            callable: new(Array.AsReadOnly(method.Parameters.Select(parameter => new Analysis.SemanticParameter(parameter.Name, Type(parameter.Type), parameter.Mode)).ToArray()),
                Type(method.Result), Array.AsReadOnly(method.GenericParameters.Select(parameter => Type(parameter.Type)).ToArray())));
        Parameters(method.Parameters, declaration.Parameters, source, symbols[method].Metadata);
    }

    internal void Parameters(IReadOnlyList<ParameterSymbol> parameters, IReadOnlyList<ParameterSyntax> syntax, SourceText source, Analysis.MetadataIdentity? method = null)
    {
        for (var index = 0; index < parameters.Count; index++)
            Declare(parameters[index], "parameter", parameters[index].Name, source, syntax[index].Name, parameters[index].Type,
                method is null ? null : method with { Kind = "parameter", Name = method.Name + "#" + index });
    }

    private void GenericParameters(IReadOnlyList<GenericParameterSymbol> parameters, IReadOnlyList<Token> syntax, SourceText source, string kind)
    {
        for (var index = 0; index < Math.Min(parameters.Count, syntax.Count); index++)
        {
            parameterKinds[parameters[index]] = kind;
            Declare(parameters[index], "type_parameter", parameters[index].Name, source, syntax[index], parameters[index].Type);
            foreach (var (type, id) in typeIds.Where(pair => ReferenceEquals(pair.Key.Parameter, parameters[index])).ToArray())
                if (types.TryGetValue(id, out var descriptor)) types[id] = descriptor with
                {
                    GenericOwner = kind == "!!" ? "method" : "type", GenericIndex = type.Parameter!.Index,
                    IsValue = type.IsValue, IsReference = type.IsReference, MayBeNull = type.NeedsNullCheck
                };
        }
    }
}
