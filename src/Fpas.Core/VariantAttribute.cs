namespace Fpas.Core;

/// <summary>Identifies the versioned metadata contract of a closed FPAS variant.</summary>
/// <param name="version">The variant metadata contract version.</param>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class VariantAttribute(int version) : Attribute
{
    /// <summary>Gets the metadata contract version understood by the compiler and debugger.</summary>
    public int Version { get; } = version;
}

/// <summary>Identifies one nested case, its tag, factory, and ordered payload names.</summary>
/// <param name="tag">The case's declaration-order tag.</param>
/// <param name="name">The case name in FPAS source.</param>
/// <param name="factory">The static factory method or fieldless-case getter.</param>
/// <param name="payloadNames">Payload names in declaration order.</param>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class VariantCaseAttribute(int tag, string name, string factory, string[] payloadNames) : Attribute
{
    /// <summary>Gets the case's declaration-order tag.</summary>
    public int Tag { get; } = tag;
    /// <summary>Gets the FPAS case name.</summary>
    public string Name { get; } = name;
    /// <summary>Gets the CLR factory method or property getter name.</summary>
    public string Factory { get; } = factory;
    /// <summary>Gets the ordered payload labels used by cross-assembly matching.</summary>
    public string[] PayloadNames { get; } = payloadNames;
}
