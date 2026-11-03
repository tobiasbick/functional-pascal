namespace Fpas.Core;

[AttributeUsage(AttributeTargets.Class | AttributeTargets.Struct, Inherited = false)]
public sealed class DomainAttribute(int version, string primitiveType, string? lowerBound, string? upperBound) : Attribute
{
    public int Version { get; } = version;
    public string PrimitiveType { get; } = primitiveType;
    public string? LowerBound { get; } = lowerBound;
    public string? UpperBound { get; } = upperBound;
}
