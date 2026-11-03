using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void ValidateInterfaceContracts(ReferenceCatalog references)
    {
        foreach (var contract in Units.SelectMany(unit => unit.Interfaces.Values))
        {
            var methods = references.InterfaceRequirements([contract.Type], contract.Owner.Source, contract.Syntax.Name.Position, validateImplementation: false)
                .Select(requirement => requirement.Method).ToArray();
            TypeSymbol Signature(TypeSymbol type) => references.SourceSignature(type, contract.GenericParameters).Erased;
            string Key(ImportedMethod method) => method.Name + ":" + (method.MethodArguments?.Count ?? 0) + ":" +
                string.Join(';', method.DelegateParameters.Select(Signature));
            foreach (var group in methods.GroupBy(Key))
            {
                var first = group.First();
                foreach (var other in group.Skip(1))
                    if (Signature(first.Result) != Signature(other.Result) || references.InterfaceGetter(first) != references.InterfaceGetter(other) ||
                        !references.CompatibleInterfaceConstraints(first, other) ||
                        !first.Parameters.Select((_, index) => first.Parameter(index).Mode == other.Parameter(index).Mode).All(equal => equal))
                        throw contract.Owner.Source.Error("FP3029", "Inherited interface slots have incompatible member kinds, results, parameter modes, or generic constraints.", contract.Syntax.Name.Position);
            }
            foreach (var method in methods)
                if (methods.Any(other => references.InterfaceGetter(other) && other.Name == "get_" + method.Name))
                    throw contract.Owner.Source.Error("FP3029", "Interface methods and properties cannot share a name.", contract.Syntax.Name.Position);
        }
    }
}
