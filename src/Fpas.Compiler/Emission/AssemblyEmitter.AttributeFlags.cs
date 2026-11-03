using System.Reflection;
using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal sealed partial class AssemblyEmitter
{
    private static bool AttributeIs(BoundAttribute attribute, string name) => attribute.Constructor.Owner.Definition?.FullName == name;

    // Serializable is a CLR type flag, not a CustomAttribute row.
    private static TypeAttributes SourceTypeFlags(IReadOnlyList<BoundAttribute> attributes)
        => attributes.Any(attribute => AttributeIs(attribute, "System.SerializableAttribute")) ? (TypeAttributes)0x2000 : 0;

    private static MethodImplAttributes SourceMethodFlags(IReadOnlyList<BoundAttribute>? attributes)
    {
        var attribute = attributes?.SingleOrDefault(attribute => AttributeIs(attribute, "System.Runtime.CompilerServices.MethodImplAttribute"));
        return attribute?.Arguments is { Count: > 0 } values ? (MethodImplAttributes)Convert.ToInt32(values[0].Value) : MethodImplAttributes.IL;
    }
}
