namespace Fpas.Core;

/// <summary>Identifies the versioned metadata contract of a closed FPAS union.</summary>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class UnionAttribute(int version) : Attribute
{
    public int Version { get; } = version;
}

/// <summary>Identifies one nested case, its tag, factory, and ordered payload names.</summary>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class UnionCaseAttribute(int tag, string name, string factory, string[] payloadNames) : Attribute
{
    public int Tag { get; } = tag;
    public string Name { get; } = name;
    public string Factory { get; } = factory;
    public string[] PayloadNames { get; } = payloadNames;
}
