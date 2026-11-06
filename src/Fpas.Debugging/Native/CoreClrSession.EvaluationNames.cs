using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed partial class EvaluationContext
    {
        public DebugBinding? Name(string name)
        {
            if (names.TryGetValue(name, out var existing)) return existing;
            var value = Find(start => session.FrameBindings(frame, start, 128, "source"), name);
            if (value is null)
            {
                var context = session.frames[frame]; using var function = context.Frame.Object(5); using var module = function.Object(3);
                var image = session.MetadataFor(module); var token = context.Symbols?.AsyncMethod(context.Data.MethodToken)?.Kickoff ?? context.Data.MethodToken;
                var owner = image.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token)).GetDeclaringType();
                for (var depth = 0; !owner.IsNil; depth++)
                {
                    if (depth == 64) throw new DebugException("limit_exceeded", "Static source scopes exceed 64 declaring types.");
                    MetadataRead();
                    using var runtimeClass = module.Object(11, (uint)MetadataTokens.GetToken(owner));
                    using var type = session.DefinitionType(runtimeClass, context.Frame);
                    value = Find(start => session.ReadStatics(type, frame, start, 128, frameType: true), name);
                    if (value is not null) break;
                    owner = image.Metadata.GetTypeDefinition(owner).GetDeclaringType();
                }
            }
            names.Add(name, value); return value;
        }

        public DebugBinding Static(string typeName, string member)
        {
            var name = typeName switch
            {
                "bool" => "Boolean", "char" => "Char", "sbyte" => "SByte", "byte" => "Byte", "short" => "Int16", "ushort" => "UInt16",
                "int" => "Int32", "uint" => "UInt32", "long" => "Int64", "ulong" => "UInt64", "nint" => "IntPtr", "nuint" => "UIntPtr",
                "float" => "Single", "double" => "Double", "decimal" => "Decimal", "string" => "String", "object" => "Object", _ => null
            };
            using var type = session.FindStaticType(name is null ? typeName : "System." + name, MetadataRead);
            using var owner = type.Object(4); using var module = owner.Object(3); var image = session.MetadataFor(module);
            var own = OwnScope(owner.UInt32(4), module);
            var accessible = image.Fields(owner.UInt32(4)).Take(4097).Where(field => field.Name == member).Any(field => own ||
                (image.Metadata.GetFieldDefinition((FieldDefinitionHandle)MetadataTokens.Handle((int)field.Token)).Attributes & FieldAttributes.FieldAccessMask) == FieldAttributes.Public);
            if (!accessible) throw new DebugException("unresolved_name", "The qualified static field is absent or inaccessible.");
            return Find(start => session.ReadStatics(type, frame, start, 128), member)
                ?? throw new DebugException("unresolved_name", "The qualified name does not identify a stored static field.");
        }

        private bool OwnScope(uint type, NativeObject module)
        {
            var context = session.frames[frame]; using var function = context.Frame.Object(5); using var currentModule = function.Object(3);
            if (!currentModule.SameIdentity(module.Pointer)) return false;
            var image = session.MetadataFor(currentModule); var token = context.Symbols?.AsyncMethod(context.Data.MethodToken)?.Kickoff ?? context.Data.MethodToken;
            var owner = image.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token)).GetDeclaringType();
            for (var depth = 0; !owner.IsNil; depth++)
            {
                if (depth == 64) throw new DebugException("limit_exceeded", "Static source scopes exceed 64 declaring types.");
                MetadataRead(); if ((uint)MetadataTokens.GetToken(owner) == type) return true;
                owner = image.Metadata.GetTypeDefinition(owner).GetDeclaringType();
            }
            return false;
        }
    }
}
