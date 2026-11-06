using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Runtime.InteropServices;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static readonly Guid class2Id = new("B008EA8D-7AB1-43F7-BB20-FBB5A04038AE");
    private static readonly Guid ilFrame2Id = new("5D88A994-6C30-479B-890F-BCEF88B129A5");

    private static unsafe NativeObject ParameterizedType(NativeObject owner, uint kind, IReadOnlyList<NativeObject> arguments)
    {
        using var extended = owner.Query(class2Id);
        var pointers = arguments.Select(argument => argument.Pointer).ToArray(); nint result = 0;
        fixed (nint* parameters = pointers)
            NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, uint, nint*, nint*, int>)extended.Slot(3))
                (extended.Pointer, kind, (uint)pointers.Length, parameters, &result));
        return new(result);
    }

    private NativeObject DefinitionType(NativeObject owner, NativeObject? frame = null)
    {
        using var module = owner.Object(3); var metadata = MetadataFor(module);
        var definition = metadata.Metadata.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.Handle((int)owner.UInt32(4)));
        var count = definition.GetGenericParameters().Count;
        if (count > 64) throw new DebugException("limit_exceeded", "The declaring type has more than 64 generic arguments.");
        if (count != 0 && frame is null) throw new DebugException("unavailable", "Use a constructed value or its active frame for generic static fields.");
        var arguments = new List<NativeObject>();
        try
        {
            if (count != 0)
            {
                using var extended = frame!.Query(ilFrame2Id); using var enumeration = extended.Object(4);
                for (var index = 0; index < count; index++)
                    arguments.Add(enumeration.Next() ?? throw new DebugException("unavailable", "The frame's declaring type arguments are unavailable."));
            }
            var kind = 18u;
            if (definition.BaseType.Kind == HandleKind.TypeReference)
            {
                var parent = metadata.Metadata.GetTypeReference((TypeReferenceHandle)definition.BaseType);
                if (metadata.Metadata.StringComparer.Equals(parent.Namespace, "System")
                    && (metadata.Metadata.StringComparer.Equals(parent.Name, "ValueType") || metadata.Metadata.StringComparer.Equals(parent.Name, "Enum"))) kind = 17;
            }
            return ParameterizedType(owner, kind, arguments);
        }
        finally { foreach (var argument in arguments) argument.Dispose(); }
    }

    private NativeObject FrameOwnerType(NativeObject frame)
    {
        using var function = frame.Object(5); using var owner = function.OptionalObject(4);
        if (owner is null) throw new DebugException("unavailable", "The frame has no declaring type.");
        return DefinitionType(owner, frame);
    }
}
