using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.References;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void DeclareRecordInterfaces(ReferenceCatalog references)
    {
        foreach (var record in Units.SelectMany(unit => unit.Records.Values))
        {
            var source = record.Owner.Source;
            var scope = record.GenericParameters.ToDictionary(parameter => parameter.Name, StringComparer.Ordinal);
            foreach (var syntax in record.Syntax.Interfaces ?? [])
            {
                var type = ResolveType(syntax, record.Owner, references, scope, validate: false);
                if (type.IsNullable || !type.IsInterface)
                    throw source.Error("FP3029", "implements requires a non-nullable interface.", syntax.Position);
                if (record.Interfaces.Any(other => other.Erased == type.Erased))
                    throw source.Error("FP3029", "An interface may occur only once in a record header.", syntax.Position);
                record.Interfaces.Add(type);
            }
        }
    }

    private void BindRecordInterfaces(ReferenceCatalog references)
    {
        foreach (var record in Units.SelectMany(unit => unit.Records.Values))
        {
            var source = record.Owner.Source;
            for (var index = 0; index < record.Interfaces.Count; index++)
                references.ValidateGenericType(record.Interfaces[index], source, record.Syntax.Interfaces![index].Position);
            foreach (var requirement in references.InterfaceRequirements(record.Interfaces, source, record.Syntax.Name.Position))
            {
                var required = requirement.Method;
                var implementations = record.Methods.Where(method => method.Name == required.Name && references.InterfaceSignature(method, record.Type, required)).ToArray();
                if (implementations.Length == 0 && references.Methods(TypeSymbol.Object, required.Name, false, required.MethodArguments).Any(method =>
                    method.Result.Erased == required.Result.Erased && method.DelegateParameters.Select(parameter => parameter.Erased).SequenceEqual(required.DelegateParameters.Select(parameter => parameter.Erased)))) continue;
                if (implementations.Length == 0 && !requirement.Required) continue;
                if (implementations.Length != 1)
                    throw source.Error("FP3029", $"Record '{record.Name}' must implement '{required.Owner}.{required.Name}' with a public instance member matching the exact CLR result, parameter types, generic arity, and modes.", record.Syntax.Name.Position);
                var body = implementations[0];
                if (body.GenericParameters.Count != 0 && !references.InterfaceConstraints(body, required))
                    throw source.Error("FP3029", "Generic implementation constraints must match the interface method constraints.", record.Syntax.Name.Position);
                record.InterfaceMethods.Add((required, body));
            }
        }
    }

}
