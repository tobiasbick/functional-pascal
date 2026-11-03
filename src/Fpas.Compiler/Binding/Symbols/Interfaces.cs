using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Symbols;

internal sealed class InterfaceSymbol(SourceUnit owner, InterfaceSyntax syntax)
{
    internal SourceUnit Owner { get; } = owner;
    internal InterfaceSyntax Syntax { get; } = syntax;
    internal string Name => Syntax.Name.Text;
    internal bool IsPublic => Syntax.IsPublic;
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters { get; set; } = [];
    internal TypeSymbol Type => new(TypeKind.Interface, Interface: this,
        Arguments: GenericParameters.Count == 0 ? null : new(GenericParameters.Select(parameter => parameter.Type)));
    internal List<TypeSymbol> Bases { get; } = [];
    internal List<RoutineSymbol> Methods { get; } = [];
    internal List<InterfacePropertySymbol> Properties { get; } = [];
    internal IReadOnlyList<BoundAttribute> Attributes { get; set; } = [];
    internal Dictionary<RoutineSymbol, IReadOnlyList<BoundAttribute>> MethodAttributes { get; } = [];
}

internal sealed class InterfacePropertySymbol(InterfacePropertySyntax syntax, RoutineSymbol getter)
{
    internal InterfacePropertySyntax Syntax { get; } = syntax;
    internal RoutineSymbol Getter { get; } = getter;
    internal IReadOnlyList<BoundAttribute> Attributes { get; set; } = [];
}
