using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using System.Xml.Linq;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Documentation;

internal sealed class DocumentationWriter
{
    private readonly List<XElement> members = [];
    private readonly List<CompilerDiagnostic> diagnostics = [];

    internal static DocumentationModel Analyze(SourceCatalog catalog, BoundCompilation compilation)
    {
        var writer = new DocumentationWriter();
        foreach (var unit in catalog.Units) writer.Unit(unit);
        foreach (var unit in compilation.Units)
            foreach (var routine in unit.Routines.Concat(unit.Records.SelectMany(record => record.Bodies.Values)).Concat(unit.Closures.SelectMany(closure => closure.Bodies)))
                foreach (var local in routine.Locals)
                    writer.Add(routine.Source!, local.Documentation, null, [], [], false);
        foreach (var unit in compilation.Units)
            foreach (var routine in unit.Closures.SelectMany(closure => closure.NestedBodies))
                writer.Routine(catalog.Units.Single(source => source.Syntax.Name.Text == unit.Name), routine.Symbol, false);
        return new(writer.members, writer.diagnostics);
    }

    private void Unit(SourceUnit unit)
    {
        unit.Source.CheckCancellation();
        var name = unit.Syntax.Name.Text;
        Add(unit.Source, unit.Syntax.Documentation, "T:" + name, [], [], false);
        foreach (var method in unit.Routines.Values.SelectMany(group => group)) Routine(unit, method, method.IsPublic);
        foreach (var global in unit.Globals.Values)
            Add(unit.Source, global.Syntax.Binding.Documentation, global.Syntax.IsPublic ? "F:" + name + "." + global.Name : null, [], [], false);
        foreach (var alias in unit.Aliases.Values) Add(unit.Source, alias.Syntax.Documentation, null, [], [], false);
        foreach (var scalar in unit.Scalars.Values)
        {
            var owner = DocumentationIds.Definition(scalar.Type);
            Add(unit.Source, scalar.Syntax.Documentation, scalar.IsPublic ? "T:" + owner : null, [], [], false);
            foreach (var member in scalar.Syntax.Members)
                Add(unit.Source, member.Documentation, scalar.IsPublic ? "F:" + owner + "." + member.Name.Text : null, [], [], false);
        }
        foreach (var record in unit.Records.Values)
        {
            var owner = DocumentationIds.Definition(record.Type);
            Add(unit.Source, record.Syntax.Documentation, record.IsPublic ? "T:" + owner : null, [], record.Syntax.TypeParameters ?? [], false);
            foreach (var field in record.Fields)
                Add(unit.Source, field.Syntax.Documentation, record.IsPublic && field.Syntax.IsPublic ? "P:" + owner + "." + field.Name : null, [], [], false);
            foreach (var method in record.UserMethods.Values.SelectMany(group => group)) Routine(unit, method, record.IsPublic && method.IsPublic);
        }
        foreach (var contract in unit.Interfaces.Values)
        {
            var owner = DocumentationIds.Definition(contract.Type);
            Add(unit.Source, contract.Syntax.Documentation, contract.IsPublic ? "T:" + owner : null, [], contract.Syntax.TypeParameters, false);
            foreach (var property in contract.Properties)
                Add(unit.Source, property.Syntax.Documentation, contract.IsPublic ? "P:" + owner + "." + property.Syntax.Name.Text : null, [], [], false);
            foreach (var method in contract.Methods.Where(method => method.Declaration is not null)) Routine(unit, method, contract.IsPublic);
        }
        foreach (var variant in unit.Variants.Values)
        {
            var owner = DocumentationIds.Definition(variant.Type);
            Add(unit.Source, variant.Syntax.Documentation, variant.IsPublic ? "T:" + owner : null, [], variant.Syntax.TypeParameters ?? [], false);
            foreach (var member in variant.Cases)
                Add(unit.Source, member.Syntax.Documentation, !variant.IsPublic ? null : member.Syntax.Fields.Count == 0 ? "P:" + owner + "." + member.Syntax.Name.Text :
                    DocumentationIds.Method(name, member.Factory), member.Syntax.Fields.Select(field => field.Name).ToArray(), [], false, parametersAllowed: member.Syntax.Fields.Count != 0);
        }
    }

    private void Routine(SourceUnit unit, RoutineSymbol method, bool visible)
    {
        var syntax = method.Declaration!;
        Add(unit.Source, syntax.Documentation, visible ? DocumentationIds.Method(unit.Syntax.Name.Text, method) : null,
            syntax.Parameters.Select(parameter => parameter.Name).ToArray(), syntax.TypeParameters ?? [], syntax.ResultType is not null, parametersAllowed: true);
    }

    private void Add(SourceText source, DocumentationComment? documentation, string? id, IReadOnlyList<Token> parameters, IReadOnlyList<Token> typeParameters,
        bool returnsAllowed, bool parametersAllowed = false)
    {
        if (documentation is null) return;
        source.CheckCancellation();
        Dictionary<string, string> Names(IReadOnlyList<Token> tokens) => tokens.ToDictionary(token => source.Text.Substring(token.Position, token.Length), token => token.Text, StringComparer.Ordinal);
        var content = StructuredDocumentation.Parse(documentation, Names(parameters), Names(typeParameters), parametersAllowed, typeParameters.Count != 0, returnsAllowed,
            (message, position) => diagnostics.Add(new("FP6001", message, source.Location(position), DiagnosticSeverity.Warning)), source.CancellationToken);
        if (id is not null) members.Add(new("member", new XAttribute("name", id), content));
    }
}
