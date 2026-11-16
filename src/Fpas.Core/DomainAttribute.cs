namespace Fpas.Core;

/// <summary>Describes an FPAS domain type: its primitive representation and optional inclusive bounds.</summary>
/// <param name="version">The domain metadata contract version.</param>
/// <param name="primitiveType">The primitive type that represents the domain.</param>
/// <param name="lowerBound">The inclusive lower bound as invariant text, or <see langword="null"/> when unbounded.</param>
/// <param name="upperBound">The inclusive upper bound as invariant text, or <see langword="null"/> when unbounded.</param>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Struct, Inherited = false)]
public sealed class DomainAttribute(int version, string primitiveType, string? lowerBound, string? upperBound) : Attribute
{
    /// <summary>Gets the domain metadata contract version.</summary>
    public int Version { get; } = version;
    /// <summary>Gets the primitive type that represents the domain.</summary>
    public string PrimitiveType { get; } = primitiveType;
    /// <summary>Gets the inclusive lower bound, or <see langword="null"/> when unbounded.</summary>
    public string? LowerBound { get; } = lowerBound;
    /// <summary>Gets the inclusive upper bound, or <see langword="null"/> when unbounded.</summary>
    public string? UpperBound { get; } = upperBound;
}
