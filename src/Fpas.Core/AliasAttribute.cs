namespace Fpas.Core;

[AttributeUsage(AttributeTargets.Method, Inherited = false)]
public sealed class AliasAttribute(int version, string name) : Attribute
{
    public int Version { get; } = version;
    public string Name { get; } = name;
}
