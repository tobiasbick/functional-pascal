using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using System.Numerics;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Symbols;

internal sealed class AliasSymbol(SourceUnit owner, AliasSyntax syntax)
{
    internal SourceUnit Owner { get; } = owner;
    internal AliasSyntax Syntax { get; } = syntax;
    internal TypeSymbol? Target { get; set; }
    internal bool Resolving { get; set; }
}

internal sealed class ScalarSymbol(SourceUnit owner, ScalarSyntax syntax)
{
    internal SourceUnit Owner { get; } = owner;
    internal ScalarSyntax Syntax { get; } = syntax;
    internal IReadOnlyList<BoundAttribute> Attributes { get; set; } = [];
    internal string Name => Syntax.Name.Text;
    internal bool IsPublic => Syntax.IsPublic;
    internal TypeSymbol Type => new(TypeKind.Scalar, Scalar: this);
    internal TypeSymbol Base { get; set; } = TypeSymbol.Unsupported;
    internal BigInteger? Lower { get; set; }
    internal BigInteger? Upper { get; set; }
    internal Action? InitializeBounds { get; set; }
    private bool resolvingBounds;
    internal void EnsureBounds()
    {
        if (InitializeBounds is null) return;
        if (resolvingBounds) throw new InvalidOperationException("Subrange bounds cannot form a cycle.");
        resolvingBounds = true;
        try { InitializeBounds(); } finally { resolvingBounds = false; }
    }
    internal List<ScalarMember> Members { get; } = [];
}

internal sealed class ScalarMember(Token name, Func<object> initialize)
{
    internal Token Name { get; } = name;
    private object? value;
    private bool resolving;
    internal object Value
    {
        get
        {
            if (value is not null) return value;
            if (resolving) throw new InvalidOperationException("Enum constants cannot form a cycle.");
            resolving = true;
            try { return value = initialize(); } finally { resolving = false; }
        }
    }
}

internal sealed record ScalarInfo(TypeSymbol Type, TypeSymbol Base, ScalarCategory Category, BigInteger? Lower = null, BigInteger? Upper = null);
