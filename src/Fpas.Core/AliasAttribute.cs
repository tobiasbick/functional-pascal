namespace Fpas.Core;

/// <summary>Records the FPAS source name of a routine whose CLR name differs.</summary>
/// <param name="version">The alias metadata contract version.</param>
/// <param name="name">The routine name in FPAS source.</param>
[AttributeUsage(AttributeTargets.Method, Inherited = false)]
public sealed class AliasAttribute(int version, string name) : Attribute
{
    /// <summary>Gets the alias metadata contract version.</summary>
    public int Version { get; } = version;
    /// <summary>Gets the routine name in FPAS source.</summary>
    public string Name { get; } = name;
}
