using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static bool NullReference(NativeObject value)
    {
        using var reference = value.TryQuery(referenceValueId);
        return reference is not null && reference.UInt32(7) != 0;
    }

    private string RuntimeTypeName(NativeObject value)
    {
        using var type = ExactType(value); using var runtimeClass = type.Object(4); using var module = runtimeClass.Object(3);
        return MetadataFor(module).TypeName(runtimeClass.UInt32(4));
    }

    private NativeObject? StoredField(NativeObject value, string name, Func<string, bool>? owner = null)
    {
        using var instance = ObjectValue(value);
        var type = ExactType(value);
        try
        {
            for (var depth = 0; depth < 64; depth++)
            {
                using var runtimeClass = type.Object(4); using var module = runtimeClass.Object(3);
                var metadata = MetadataFor(module); var token = runtimeClass.UInt32(4);
                if (owner is null || owner(metadata.TypeName(token)))
                {
                    var field = metadata.Fields(token).FirstOrDefault(field => field.Name == name);
                    if (field.Token != 0) return instance.Field(runtimeClass, field.Token);
                }
                var next = type.OptionalObject(7);
                if (next is null) return null;
                type.Dispose(); type = next;
            }
            throw new DebugException("limit_exceeded", "Stored-field lookup exceeds 64 base types.");
        }
        finally { type.Dispose(); }
    }
}
