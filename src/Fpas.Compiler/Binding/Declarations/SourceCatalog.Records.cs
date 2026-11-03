using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void DeclareRecords(ReferenceCatalog references)
    {
        foreach (var unit in Units)
        {
            foreach (var syntax in unit.Syntax.Records)
            {
                if (TypeFacts.PredefinedName(syntax.Name.Text) || unit.Unions.ContainsKey(syntax.Name.Text) || unit.Syntax.Imports.Any(import => import.Alias.Text == syntax.Name.Text) || !unit.Records.TryAdd(syntax.Name.Text, new(unit, syntax)))
                    throw unit.Source.Error("FP3021", $"Duplicate or conflicting record '{syntax.Name.Text}'.", syntax.Name.Position);
            }
        }
        DeclareScalarBases(references);
        DeclareTypeGenerics(references);
        BindScalarValues(references);
        DeclareRecordInterfaces(references);
        if (Units.Any(unit => unit.Records.Count != 0)) references.Required("Fpas.Core.RecordAttribute");
        foreach (var record in Units.SelectMany(unit => unit.RecordTypes))
            foreach (var field in record.Syntax.Fields)
                record.Fields.Add(new(field, ResolveType(field.Type, record.Owner, references,
                    record.GenericParameters.ToDictionary(parameter => parameter.Name, StringComparer.Ordinal), validate: false)));
        foreach (var record in Units.SelectMany(unit => unit.RecordTypes))
            foreach (var field in record.Fields) references.ValidateGenericType(field.Type, record.Owner.Source, field.Syntax.Name.Position);
        ValidateTypeGenerics(references);
        foreach (var record in Units.SelectMany(unit => unit.RecordTypes))
        {
            var unit = record.Owner;
            var names = new HashSet<string>(["Equals", "GetHashCode", "op_Equality", "op_Inequality"], StringComparer.Ordinal);
            foreach (var field in record.Syntax.Fields)
            {
                if (!names.Add(field.Name.Text) || !names.Add("get_" + field.Name.Text))
                    throw unit.Source.Error("FP3021", $"Record field '{field.Name.Text}' collides with another member.", field.Name.Position);
            }
            var canConstruct = record.UnionCase is null && record.Fields.All(field => field.Syntax.IsPublic);
            Add(record, ".ctor", TypeSymbol.Void, record.Fields.Select(field => (field.Name, field.Type)), canConstruct, true, GeneratedMethodKind.Constructor);
            foreach (var field in record.Fields)
            {
                field.Getter = Add(record, "get_" + field.Name, field.Type, [], field.Syntax.IsPublic, true, GeneratedMethodKind.Getter);
                if (record.UnionCase is null) field.Copy = Add(record, "<With:" + field.Name + ">", record.Type, [("Value", field.Type)], field.Syntax.IsPublic, true, GeneratedMethodKind.Copy);
                if (field.Syntax.Default is not null)
                    field.Default = Add(record, "<Default:" + field.Name + ">", field.Type, [], canConstruct, false, GeneratedMethodKind.Default);
            }
            var nullable = record.Type with { IsNullable = true };
            Add(record, "Equals", TypeSymbol.Boolean, [("Other", nullable)], true, true, GeneratedMethodKind.EqualsTyped);
            Add(record, "Equals", TypeSymbol.Boolean, [("Other", TypeSymbol.Object with { IsNullable = true })], true, true, GeneratedMethodKind.EqualsObject);
            Add(record, "GetHashCode", TypeSymbol.Int32, [], true, true, GeneratedMethodKind.Hash);
            Add(record, "op_Equality", TypeSymbol.Boolean, [("Left", nullable), ("Right", nullable)], true, false, GeneratedMethodKind.Equality);
            Add(record, "op_Inequality", TypeSymbol.Boolean, [("Left", nullable), ("Right", nullable)], true, false, GeneratedMethodKind.Inequality);
            foreach (var method in record.Syntax.Methods)
            {
                if (names.Contains(method.Name.Text)) throw unit.Source.Error("FP3021", $"Record method '{method.Name.Text}' collides with another member.", method.Name.Position);
                var symbol = DeclareRoutine(unit, method, references, -1, record);
                record.Methods.Add(symbol);
                AddOverload(record.UserMethods, symbol, unit.Source, method.Name.Position);
            }
            if (record.UnionCase is not null)
            {
                Add(record, "get_Tag", TypeSymbol.Int32, [], true, true, GeneratedMethodKind.Tag);
                continue;
            }
            record.Info = new(record.Type, record.Fields.Select(field => new RecordFieldInfo(field.Name, field.Type, field.Syntax.IsPublic,
                record.Method(field.Getter), record.Method(field.Copy!), field.Default is null ? null : record.Method(field.Default))).ToArray(), record.Method(record.Constructor), canConstruct);
        }
    }

    private static RoutineSymbol Add(RecordSymbol owner, string name, TypeSymbol result, IEnumerable<(string Name, TypeSymbol Type)> parameters,
        bool isPublic, bool instance, GeneratedMethodKind kind)
    {
        var symbol = new RoutineSymbol(name, result, parameters.Select((parameter, index) => new ParameterSymbol(parameter.Name, parameter.Type, index)).ToArray(),
            isPublic, false, -1, instance, owner, kind);
        owner.Methods.Add(symbol);
        return symbol;
    }

    private void ValidateRecordVisibility()
    {
        static bool Visible(TypeSymbol type) => type.Scalar?.IsPublic != false && type.Record?.IsPublic != false && type.Union?.IsPublic != false && (type.Element is null || Visible(type.Element)) && (type.Arguments is null || type.Arguments.All(Visible));
        void Check(SourceUnit unit, RoutineSymbol method, int position)
        {
            if (method.IsPublic && (!Visible(method.Result) || method.Parameters.Any(parameter => !Visible(parameter.Type)) || method.GenericParameters.SelectMany(parameter => parameter.Constraints).Any(type => !Visible(type))))
                throw unit.Source.Error("FP3021", "A public signature cannot expose a private record.", position);
        }
        void CheckConstraints(SourceUnit unit, IReadOnlyList<GenericParameterSymbol> parameters, int position)
        {
            if (parameters.SelectMany(parameter => parameter.Constraints).Any(type => !Visible(type)))
                throw unit.Source.Error("FP3021", "A public type cannot expose a private type through generic constraints.", position);
        }
        foreach (var unit in Units)
        {
            foreach (var union in unit.Unions.Values.Where(union => union.IsPublic))
            {
                CheckConstraints(unit, union.GenericParameters, union.Syntax.Name.Position);
                foreach (var method in union.Methods) Check(unit, method, union.Syntax.Name.Position);
            }
            foreach (var global in unit.Globals.Values.Where(global => global.Syntax.IsPublic))
                if (!Visible(global.Type)) throw unit.Source.Error("FP3032", "A public global cannot expose private types.", global.Syntax.Binding.Name.Position);
            foreach (var routine in unit.Routines.Values.SelectMany(group => group)) Check(unit, routine, unit.Syntax.Name.Position);
            foreach (var record in unit.RecordTypes.Where(record => record.IsPublic))
            {
                if (record.Interfaces.Any(type => !Visible(type))) throw unit.Source.Error("FP3021", "A public record cannot expose private types through an interface.", record.Syntax.Name.Position);
                CheckConstraints(unit, record.GenericParameters, record.Syntax.Name.Position);
                foreach (var method in record.Methods) Check(unit, method, record.Syntax.Name.Position);
            }
        }
    }
}
