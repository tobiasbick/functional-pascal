namespace Fpas.Core;

/// <summary>Identifies the versioned FPAS record metadata contract.</summary>
/// <param name="version">The record metadata contract version.</param>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class RecordAttribute(int version) : Attribute
{
    /// <summary>Gets the record metadata contract version.</summary>
    public int Version { get; } = version;
}

/// <summary>Describes a record field and its compiler-generated copy/default providers.</summary>
/// <param name="order">The declaration order of the field.</param>
/// <param name="copyMethod">The compiler-generated method that copies the record with a new field value.</param>
/// <param name="defaultProvider">The member that supplies the field default, or <see langword="null"/> without one.</param>
[AttributeUsage(AttributeTargets.Property, Inherited = false)]
public sealed class RecordFieldAttribute(int order, string copyMethod, string? defaultProvider) : Attribute
{
    /// <summary>Gets the declaration order of the field.</summary>
    public int Order { get; } = order;
    /// <summary>Gets the compiler-generated copy method name.</summary>
    public string CopyMethod { get; } = copyMethod;
    /// <summary>Gets the default provider member name, or <see langword="null"/> without one.</summary>
    public string? DefaultProvider { get; } = defaultProvider;
}
