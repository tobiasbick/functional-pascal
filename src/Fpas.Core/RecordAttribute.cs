namespace Fpas.Core;

/// <summary>Identifies the versioned FPAS record metadata contract.</summary>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class RecordAttribute(int version) : Attribute
{
    public int Version { get; } = version;
}

/// <summary>Describes a record field and its compiler-generated copy/default providers.</summary>
[AttributeUsage(AttributeTargets.Property, Inherited = false)]
public sealed class RecordFieldAttribute(int order, string copyMethod, string? defaultProvider) : Attribute
{
    public int Order { get; } = order;
    public string CopyMethod { get; } = copyMethod;
    public string? DefaultProvider { get; } = defaultProvider;
}
