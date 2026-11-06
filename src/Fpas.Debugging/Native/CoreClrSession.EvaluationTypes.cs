using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed partial class EvaluationContext
    {
        public DebugBinding MemberType(DebugBinding receiver, string name)
        {
            var identity = receiver.RuntimeType;
            if (identity?.ModuleId is not { } moduleId || identity.Token is not { } token)
            {
                if (receiver.Reference is not null) return Member(receiver, name);
                throw new DebugException("unavailable", "The declared field type's loaded module identity is unavailable.");
            }
            var module = session.modules.SingleOrDefault(item => item.Id == moduleId)
                ?? throw new DebugException("unavailable", "The field type's module is no longer loaded.");
            MetadataRead(); var image = session.MetadataFor(module.Native); var shape = image.Shape(token);
            if (shape?.Tag is not null && name == "Tag") return new("", "int", "integer", null, RuntimeType: new("int", 8));
            if (shape?.Kind == "domain" && name == "Value")
            {
                var primitive = shape.Primitive switch
                {
                    "System.Boolean" => 2u, "System.Char" => 3u, "System.SByte" => 4u, "System.Byte" => 5u, "System.Int16" => 6u, "System.UInt16" => 7u,
                    "System.Int32" => 8u, "System.UInt32" => 9u, "System.Int64" => 10u, "System.UInt64" => 11u, "System.Single" => 12u, "System.Double" => 13u,
                    "System.String" => 14u, "System.IntPtr" => 24u, "System.UIntPtr" => 25u, _ => 0u
                };
                if (shape.Primitive == "System.Decimal") return new("", "decimal", "decimal", null);
                if (primitive == 0) throw new DebugException("invalid_metadata", "Unsupported domain primitive metadata.");
                return TypeValue(new(ScalarValue.TypeName(primitive)!, primitive));
            }
            var field = shape is null ? image.Fields(token).Take(4097).FirstOrDefault(field => field.Name == name).Token
                : shape.Members.FirstOrDefault(member => member.Name == name)?.Field ?? 0;
            if (field != 0) return TypeValue(new StoredTypeSignature(image, moduleId, identity.Arguments ?? []).Field(field));
            var definition = image.Metadata.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.Handle((int)token));
            if (definition.GetProperties().Any(handle => image.Metadata.StringComparer.Equals(image.Metadata.GetPropertyDefinition(handle).Name, name)))
                throw new DebugException("requires_execution", "A custom property getter is outside passive inspection.");
            throw new DebugException("unresolved_name", $"No stored source field is named '{name}'.");
        }

        private static DebugBinding TypeValue(DebugTypeIdentity type) => new("", type.Name, type.ElementKind == 2 ? "boolean" : "unknown", null, RuntimeType: type);
    }
}
